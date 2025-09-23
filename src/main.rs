use clap::{Args, Parser};
use gatekeeper_core::{GatekeeperReader, NfcTag, Realm, RealmType};
use lazy_static::lazy_static;
use paho_mqtt as mqtt;
use serde::Serialize;
use std::collections::HashMap;
use std::panic;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod door;
mod pwm;
use crate::door::{Door, FakeDoor, ZuulDoor};

#[derive(Serialize, Debug)]
struct AccessRequested<'a> {
    association: &'a str,
}

lazy_static! {
    static ref SUPERUSERS: HashMap<String, String> = HashMap::new();
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
struct CliArgs {
    /// Device connection string (e.g. 'pn532_uart:/dev/ttyUSB0')
    device: String,

    #[allow(dead_code)]
    /// Door identifier
    #[arg(long, env = "GK_ACCESS_POINT")]
    access_point: String,
    /// Gatekeeper auth key
    #[arg(long, env = "GK_REALM_DOORS_AUTH_KEY")]
    auth_key: String,
    /// Gatekeeper read key
    #[arg(long, env = "GK_REALM_DOORS_READ_KEY")]
    read_key: String,
    /// Gatekeeper public key
    #[arg(long, env = "GK_REALM_DOORS_PUBLIC_KEY")]
    public_key: String,

    /// Username for the MQTT server
    #[arg(long, env = "GK_MQTT_USERNAME", default_value = "")]
    mqtt_username: String,
    /// Password for the MQTT server
    #[arg(long, env = "GK_MQTT_PASSWORD", default_value = "")]
    mqtt_password: String,
    /// Hostname for MQTT server
    #[arg(long, env = "GK_MQTT_SERVER")]
    mqtt_server: String,

    /// Gatekeeper mobile encryption private key
    #[arg(long, env = "GK_REALM_DOORS_MOBILE_CRYPT_PRIVATE_KEY")]
    mobile_decryption_private_key: String,
    /// Gatekeeper mobile signing private key
    #[arg(long, env = "GK_REALM_DOORS_MOBILE_PRIVATE_KEY")]
    mobile_signing_private_key: String,

    /// Door parameters
    #[command(flatten)]
    zuul: ZuulDoorParams,

    /// Simulate?
    #[arg(long)]
    simulate: bool,

    /// Unlock?
    #[arg(long)]
    unlock: bool,
}

#[derive(Args, Debug, Clone)]
struct ZuulDoorParams {
    /// GPIO pin to drive the motor forward
    #[arg(long, env = "GK_DOOR_MOTOR_F_PIN")]
    door_f_pin: u32,
    /// GPIO pin to drive the motor backward
    #[arg(long, env = "GK_DOOR_MOTOR_R_PIN")]
    door_r_pin: u32,
    /// GPIO pin attached to the LED
    #[arg(long, env = "GK_DOOR_LED_PIN")]
    door_led_pin: u32,
    /// GPIO chip path
    #[arg(long, env = "GK_DOOR_GPIO_CHIP", default_value = "/dev/gpiochip0")]
    door_gpio_chip: PathBuf,

    /// PWM pin attached to the buzzer
    #[arg(long, env = "GK_DOOR_BUZZER_PIN", default_value = "0")]
    door_buzzer_pin: usize,
    /// PWM chip id
    #[arg(long, env = "GK_DOOR_BUZZER_CHIP", default_value = "0")]
    door_buzzer_chip: usize,
}

impl CliArgs {
    fn get_prefix(&self) -> String {
        let Self { access_point, .. } = self;
        format!("gk/{access_point}")
    }
}

fn main() {
    dotenvy::dotenv().ok();
    // Configure Logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("gatekeeper_door=info"),
    )
    .init();

    // take_hook() returns the default hook in case when a custom one is not set
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        process::exit(1);
    }));

    // Parse arguments
    let args = CliArgs::parse();

    // Run the Gatekeeper client
    match &args.simulate {
        false => run(
            ZuulDoor::new(
                &args.zuul.door_gpio_chip,
                args.zuul.door_r_pin,
                args.zuul.door_f_pin,
                args.zuul.door_led_pin,
                args.zuul.door_buzzer_chip,
                args.zuul.door_buzzer_pin,
            ),
            args,
        ),
        true => run(FakeDoor, args),
    }
}

fn check_mqtt<T: Door + Send>(client: mqtt::AsyncClient, door: &Mutex<T>, args: CliArgs) {
    let prefix = args.get_prefix();
    let prefix_trailing = format!("{prefix}/");
    let mut attempt = 0;

    loop {
        log::info!("Running a tick of the MQTT check loop (This shouldn't happen often!)");
        let mqtt_queue = client.start_consuming();

        if attempt >= 10 {
            log::error!("Failed to subscribe after 10 attempts, giving up!");
            break;
        }

        // logs the error instead of failing silently
        // wont panic and crash the program 
        if let Err(err) = client.subscribe_many(  
            &[
                format!("{prefix_trailing}{REMOTE_UNLOCK}"),
                format!("{prefix_trailing}{ACCESS_DENIED}"),
            ],
            &[1, 1],
        ).wait() {
            log::warn!("Subscribe failed (attempt {}): {}",attempt + 1, err);
            attempt += 1;
            continue;
        }

        attempt = 0; // reset attempts

        for msg in mqtt_queue.iter() {
            if let Some(msg) = msg {
                let topic = msg.topic().strip_prefix(&prefix_trailing);
                match topic {
                    Some(REMOTE_UNLOCK) => {
                        door.lock().unwrap().access_granted();
                    }
                    Some(ACCESS_DENIED) => {
                        door.lock().unwrap().access_denied();
                    }
                    Some(topic) => {
                        log::warn!("Unknown topic: {topic}");
                    }
                    None => {
                        log::info!("Throwing away a message missing prefix {prefix_trailing} (should have trailing /): {}", msg.topic());
                    }
                }
            } else {
                // Shouldn't be necessary but who knows :shrug:
                log::warn!("Reconnecting. Are we connected? {}", client.is_connected());
                if let Err(err) = client.reconnect().wait() {
                    log::warn!("Failed to reconnect, retrying: {}", err);
                }
                break;
            }
        }
    }
}

fn door_heartbeat(client: mqtt::AsyncClient, args: &CliArgs) {
    let prefix = args.get_prefix();
    let heartbeat = format!("{prefix}/heartbeat");
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

const REMOTE_UNLOCK: &str = "unlock";
const ACCESS_DENIED: &str = "access_denied";

fn check_available_tags<T: Door + Send>(
    just_scanned: bool,
    args: &CliArgs,
    gatekeeper_reader: &mut GatekeeperReader,
    door: &Mutex<T>,
    client: &mqtt::AsyncClient,
) -> Result<bool, Box<dyn std::error::Error>> {
    let now = Instant::now();
    if let Some(association) = gatekeeper_reader
        .get_nearby_tags()
        .into_iter()
        .next()
        .map(|tag| tag.authenticate())
    {
        let association = association?;
        // User hasn't taken their tag off the reader yet, don't spam requests
        if just_scanned {
            return Ok(true);
        }
        log::info!(
            "We appear to be reading a valid key ({}), let's tell the server!",
            association
        );
        log::debug!("Took us {}ms to read!", now.elapsed().as_millis());
        if SUPERUSERS.contains_key(&association) {
            door.lock().unwrap().access_granted();
        } else {
            // Yay!
            let payload = serde_json::to_string(&AccessRequested {
                association: &association,
            })?;
            let msg = mqtt::Message::new(
                format!("{}/access_requested", args.get_prefix()),
                payload,
                mqtt::QOS_1,
            );
            // NB: we're using AsyncClient, so there's no need for another thread here :)
            // (Because we don't want to freeze trying to pub)
            let delivery_token = client.publish(msg);
            std::thread::spawn(move || match delivery_token.wait() {
                Ok(()) => log::debug!("Message published!"),
                Err(err) => {
                    log::error!("Couldn't publish a message: {err}");
                }
            });
        }
        Ok(true)
    } else {
        // No key, we have no longer just scanned!
        Ok(false)
    }
}

fn run<T: Door + Send + 'static>(door: T, args: CliArgs) {
    let door = Arc::new(Mutex::new(door));
    let realm = Realm::new(
        RealmType::Door,
        args.auth_key.clone().into_bytes(),
        args.read_key.clone().into_bytes(),
        args.public_key.as_bytes(),
        args.mobile_decryption_private_key.as_bytes(),
        args.mobile_signing_private_key.as_bytes(),
        None,
    );
    let mut gatekeeper_reader = GatekeeperReader::new(args.device.clone(), realm)
        .ok_or("failed to get gatekeeper device")
        .unwrap();

    let client = mqtt::AsyncClient::new(args.mqtt_server.as_str()).unwrap();
    log::info!(
        "Connecting to MQTT at {} using {} : {}",
        args.mqtt_server,
        args.mqtt_username,
        args.mqtt_password
    );
    match client
        .connect(
            mqtt::connect_options::ConnectOptionsBuilder::new_ws()
                .keep_alive_interval(Duration::from_secs(30))
                .user_name(args.mqtt_username.clone())
                .password(args.mqtt_password.clone())
                .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30))
                .ssl_options(mqtt::ssl_options::SslOptions::new())
                .finalize(),
        )
        .wait()
    {
        Ok(_) => {
            println!("Established connection with broker!");
        }
        Err(err) => {
            println!("Couldn't connect to MQTT broker! {}", err);
        }
    }

    {
        let client = client.clone();
        let args = args.clone();
        let door = door.clone();
        thread::spawn(move || check_mqtt(client, &*door, args));
    }

    {
        let client = client.clone();
        let args = args.clone();
        thread::spawn(move || door_heartbeat(client, &args));
    }

    // Wait until the tag disappears before re-scanning:
    let mut just_scanned = false;

    let door = &*door;
    if args.unlock {
        door.lock().unwrap().access_granted();
        return;
    }
    loop {
        match check_available_tags(just_scanned, &args, &mut gatekeeper_reader, door, &client) {
            Ok(found_a_tag) => {
                just_scanned = found_a_tag;
            }
            Err(err) => {
                log::error!("Couldn't scan a tag: {err}");
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
}
