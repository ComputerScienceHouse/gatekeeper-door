#[macro_use]
extern crate log;
#[macro_use]
extern crate chan;
extern crate libgatekeeper_sys;
extern crate serde_json;
extern crate paho_mqtt as mqtt;

use clap::{App, Arg, ArgMatches};
use chan_signal::Signal;
use log::LogLevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use libgatekeeper_sys::{Nfc, Realm};
use serde_json::json;
use std::env;
use std::time::Duration;
use std::thread;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::sync::Arc;


use gatekeeper::beeper::Beeper;

#[derive(Clone)]
struct Provisions {
    access_point: String,
    auth_key: String,
    read_key: String,
    update_key: String,
    public_key: String,
    private_key: String,
    prefix: String,

    // MQTT creds
    mqtt_username: String,
    mqtt_password: String
}

// const QOS: &[i32] = &[1, 1];

fn main() {
    // Configure Logging
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LogLevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();

    // Parse arguments
    let matches = App::new("Gatekeeper Door")
        .version("0.1.0")
        .author("Steven Mirabito <steven@stevenmirabito.com>")
        .about("Door lock client software for the Gatekeeper access control system")
        .arg(Arg::with_name("DEVICE")
             .help("Device connection string (e.g. 'pn532_uart:/dev/ttyUSB0')")
             .required(true)
             .index(1))
        .get_matches();

    // Handle signals when the OS sends an INT or TERM
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (sdone, rdone) = chan::sync(0);

    // _keys are stored in env vars because it seems like the best place to keep secrets
    // We panic because there's no sensible default for this...
    let access_point = env::var("GK_ACCESS_POINT").unwrap().to_string();
    let provisions = Provisions {
        auth_key: env::var("GK_AUTH_KEY").unwrap_or("dead".to_string()),
        read_key: env::var("GK_READ_KEY").unwrap_or("beef".to_string()),
        update_key: env::var("GK_UPDATE_KEY").unwrap_or("f00".to_string()),
        public_key: env::var("GK_PUBLIC_KEY").unwrap_or("face".to_string()),
        private_key: env::var("GK_PRIVATE_KEY").unwrap_or("cafe".to_string()),
        prefix: "gk/".to_string() + &access_point.clone(),
        access_point,

        mqtt_password: env::var("GK_MQTT_PASSWORD").unwrap_or("".to_string()),
        mqtt_username: env::var("GK_MQTT_USERNAME").unwrap_or("".to_string()),
    };

    // Run the Gatekeeper client
    thread::spawn(|| { run(sdone, matches, provisions) });

    // Wait for a signal
    chan_select! {
        signal.recv() -> signal => {
            info!("Received SIG{:?}, shutting down", signal.unwrap());
        },
        rdone.recv() => {
            info!("Reached exit condition, shutting down");
        }
    }
}

enum RequestType {
    FetchUser,
    DoorCheck
}

struct Request {
    request_type: RequestType,
    payload: String
}

fn check_mqtt(
    mut client: mqtt::AsyncClient, beeper: &Option<Beeper>,
    provisions: Provisions, tx: std::sync::mpsc::Sender<String>
) {
    let fetch_user = provisions.prefix.clone() + "/fetch_user";
    let access_requested = provisions.prefix.clone() + "/access_requested";

    let mqtt_queue = client.start_consuming();

    let remote_unlock = provisions.prefix.clone() + "/unlock";
    let user_response = provisions.prefix.clone() + "/user_response";
    let access_denied = provisions.prefix.clone() + "/access_denied";
    // We unwrap not because we need the response, but because we want to make sure
    // We aren't writing buggy code!
    client.subscribe_many(&[
        remote_unlock.clone(),
        user_response.clone(),
        access_denied.clone()
    ], &[1, 1, 1]).wait().unwrap();

    loop {
        for msg in mqtt_queue.iter() {
            if let Some(msg) = msg {
                if msg.topic() == user_response {
                    tx.send(String::from_utf8(msg.payload().to_vec()).unwrap());
                } else if msg.topic() == remote_unlock.clone() {
                    unlock(beeper);
                } else if msg.topic() == access_denied.clone() {
                    if let Some(ref beeper) = *beeper {
                        beeper.access_denied();
                    }
                }
            }
        }
        // Shouldn't be necessary but who knows :shrug:
        client.reconnect().wait();
    }
}

fn door_heartbeat(mut client: mqtt::AsyncClient, provisions: Provisions) {
    let heartbeat = provisions.prefix.clone() + "/heartbeat";
    loop {
        let msg = mqtt::Message::new(heartbeat.clone(), "{}", mqtt::QOS_1);
        if let Err(err) = client.publish(msg).wait() {
            println!("Couldn't publish heartbeat?! {}", err);
        } else {
            println!("Published a new heartbeat");
        }
        thread::sleep(Duration::from_secs(15));
    }
}

fn run(_sdone: chan::Sender<()>, args: ArgMatches<'_>, provisions: Provisions) {
    let beeperArc = Arc::new(Beeper::new().ok_or("failed to open beeper").ok());
    let mut nfc = Nfc::new().ok_or("failed to create NFC context").unwrap();
    let conn_str = args.value_of("DEVICE").unwrap().to_string();
    let mut device = nfc.gatekeeper_device(conn_str).ok_or("failed to get gatekeeper device").unwrap();

    let mut client = mqtt::AsyncClient::new(env::var("GK_MQTT_SERVER").unwrap()).unwrap();
    match client.connect(
        mqtt::connect_options::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(30))
            .user_name(provisions.mqtt_username.clone())
            .password(provisions.mqtt_password.clone())
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30))
            .finalize()
    ).wait() {
        Ok(_) => {
            println!("Established connection with broker!");
        }
        Err(err) => {
            println!("Couldn't connect to MQTT broker! {}", err);
        }
    }
    let beeper = beeperArc.clone();

    let (send_user, user_response) = channel::<String>();
    let mut superusers: HashMap<String, String>  = HashMap::new();
    // superusers.insert("045604da594680".to_string(), "7c5d9984-8392-4dce-8dc1-75791fa6bf31".to_string());

    {
        let client = client.clone();
        let provisions = provisions.clone();
        thread::spawn(move || { check_mqtt(client, &beeperArc.clone(), provisions, send_user) });
    }

    {
        let client = client.clone();
        let provisions = provisions.clone();
        thread::spawn(move || { door_heartbeat(client, provisions) });
    }

    // lol panic

    let access_requested = provisions.prefix.clone() + "/access_requested";
    let fetch_user = provisions.prefix.clone() + "/fetch_user";

    let slot = 0;
    let slot_name = "Doors";

    // Wait until the tag disappears before re-scanning:
    let mut just_scanned = false;

    loop {
        let result = device.first_tag();

        match result {
            Some(mut tag) => {
                if just_scanned {
                    thread::sleep(Duration::from_millis(250));
                    continue;
                }
                just_scanned = true;
                println!("Tag UID: {:?}", tag.get_uid());
                println!("Tag Name: {:?}", tag.get_friendly_name());

                let mut valid_key = false;

                let realm = Realm::new(slot, slot_name, "",
                                       &provisions.auth_key, &provisions.read_key,
                                       &provisions.update_key, &provisions.public_key,
                                       &provisions.private_key);
                if let Some(mut realm) = realm {
                    if let Ok(association) = tag.authenticate(&mut realm) {
                        println!("We appear to be reading a valid key ({}), let's tell the server!", association);
                        valid_key = true;
                        if superusers.contains_key(&tag.get_uid().unwrap()) {
                            unlock(&beeper);
                        } else {
                            // Yay!
                            let payload = json!({
                                "association": association
                            }).to_string();
                            let msg = mqtt::Message::new(
                                access_requested.clone(), payload, mqtt::QOS_1
                            );
                            // NB: we're using AsyncClient, so there's no need for another thread here :)
                            // (Because we don't want to freeze trying to pub)
                            client.publish(msg);
                        }
                    } else {
                        println!("Couldn't authenticate key! -- It's possible someone is doing something nasty!");
                    }
                }

                if !valid_key {
                    if let Some(ref beeper) = *beeper {
                        beeper.access_denied();
                    }
                }
            },
            None => {
                just_scanned = false;
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn unlock(beeper: &Option<Beeper>) {
    println!("Unlocking!");
    if let Some(beeper) = beeper {
        beeper.access_granted();
        // Do the whole unlock thing??
    }
}
