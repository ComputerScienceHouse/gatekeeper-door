extern crate serde;
extern crate serde_json;
extern crate libgatekeeper_sys;
extern crate reqwest;

use std::env;
use clap::{App, Arg};
use libgatekeeper_sys::{Nfc, Realm};
use serde_json::json;
use std::time::Duration;
use std::thread;
use std::io;
use serde::{Serialize, Deserialize};
// use reqwest::blocking::Client;
use reqwest::StatusCode;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct KeyCreated {
    keyId: String,
}

struct Provisions {
    auth_key: String,
    read_key: String,
    update_key: String,
    public_key: String,
    private_key: String,
    prefix: String,
    system_secret: String
}

fn main() {
    let matches = App::new("Gatekeeper Door")
        .version("0.1.0")
        .author("Steven Mirabito <steven@stevenmirabito.com>")
        .about("Door lock client software for the Gatekeeper access control system")
        .arg(Arg::with_name("DEVICE")
             .help("Device connection string (e.g. 'pn532_uart:/dev/ttyUSB0')")
             .required(true)
             .index(1))
        .get_matches();

    let conn_str = matches.value_of("DEVICE").unwrap().to_string();
    let mut nfc = Nfc::new().ok_or("failed to create NFC context").unwrap();
    let mut device = nfc.gatekeeper_device(conn_str).ok_or("failed to get gatekeeper device").unwrap();

    let client = reqwest::blocking::Client::new();

    let provisions = Provisions {
        auth_key: env::var("GK_AUTH_KEY").unwrap_or("dead".to_string()),
        read_key: env::var("GK_READ_KEY").unwrap_or("beef".to_string()),
        update_key: env::var("GK_UPDATE_KEY").unwrap_or("f00".to_string()),
        public_key: env::var("GK_PUBLIC_KEY").unwrap_or("face".to_string()),
        private_key: env::var("GK_PRIVATE_KEY").unwrap_or("cafe".to_string()),
        system_secret: env::var("GK_SYSTEM_SECRET").unwrap_or("b00".to_string()),
        prefix: env::var("GK_HTTP_ENDPOINT").unwrap_or("http://localhost:3000".to_string())
    };

    let slot = 0;
    let slot_name = "Doors";

    loop {
        // https://github.com/rust-lang/rust/issues/59015
        let mut username: String = "".to_string();
        println!("Enter username:");
        match io::stdin().read_line(&mut username) {
            Ok(_) => {
                // Drop newline
                username.pop();
                // TODO: Translate username => id
                println!("Ok, enrolling {}", username);
                let res_result = client.put(provisions.prefix.clone() + "/users")
                    .json(&json!({
                        "id": username
                    }))
                    .send();

                match res_result {
                    Ok(res) => match res.status() {
                        StatusCode::OK =>
                            println!("Issued for {}!", username),
                        status => {
                            println!("Failed to associate key with user! {:?}", status);
                            continue;
                        }
                    }
                    Err(error) => {
                        println!("Failed to associate key with user! {:?}", error);
                        continue;
                    }
                }


                // Now we can ask the server!
                // We unwrap here because pattern matching hell
                // + there's probably something very wrong at that point
                let res = client.put(provisions.prefix.clone() + "/keys")
                    .json(&json!({
                        "userId": username
                    }))
                    .send()
                    .unwrap();
                match res.json::<KeyCreated>() {
                    Ok(data) => {
                        let association = data.keyId;
                        // Now we can ask for the key!
                        let mut realm = Realm::new(
                            slot, slot_name, &association.clone(),
                            &provisions.auth_key, &provisions.read_key,
                            &provisions.update_key, &provisions.public_key,
                            &provisions.private_key
                        ).unwrap();
                        loop {
                            let tag = device.first_tag();
                            if let Some(mut tag) = tag {
                                match tag.issue(&provisions.system_secret.clone(), &mut realm) {
                                    Ok(_) => {
                                        let res_result = client.patch(
                                            provisions.prefix.clone() + "/keys/" + &association
                                        ).json(&json!({
                                            "enabled": true
                                        })).send();

                                        match res_result {
                                            Ok(res) => match res.status() {
                                                StatusCode::OK =>
                                                    println!("Issued for {}!", username),
                                                status => {
                                                    println!("Failed to associate key with user! {:?}", status);
                                                    continue;
                                                }
                                            },
                                            Err(error) => {
                                                println!("Failed to associate key with user! {:?}", error);
                                                continue;
                                            }
                                        }
                                        break;
                                    },
                                    Err(err) => {
                                        println!("Failed issuing... {:?}", err);
                                    }
                                }
                            }
                            thread::sleep(Duration::from_millis(200));
                        }
                    },
                    Err(err) => {
                        println!("Error creating initial key: {}", err);
                    }
                }
            }
            Err(error) => {
                println!("Sorry. Try again: {}", error);
            }
        }
    }
}
