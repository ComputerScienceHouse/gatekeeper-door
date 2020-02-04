#[macro_use]
extern crate log;
#[macro_use]
extern crate chan;
extern crate libgatekeeper_sys;

use std::thread;
use clap::{App, Arg, ArgMatches};
use chan_signal::Signal;
use log::LogLevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use libgatekeeper_sys::Nfc;

use gatekeeper::beeper::Beeper;

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

    // Run the Gatekeeper client
    thread::spawn(move || run(sdone, matches));

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

fn run(_sdone: chan::Sender<()>, args: ArgMatches) {
    let beeper = Beeper::new().ok_or("failed to open beeper").unwrap();
    let mut nfc = Nfc::new().ok_or("failed to create NFC context").unwrap();
    let conn_str = args.value_of("DEVICE").unwrap().to_string();
    let mut device = nfc.gatekeeper_device(conn_str).ok_or("failed to get gatekeeper device").unwrap();

    loop {
        info!("Polling for tag...");
        let result = device.first_tag();

        match result {
            Some(mut tag) => {
                println!("Tag UID: {:?}", tag.get_uid());
                println!("Tag Name: {:?}", tag.get_friendly_name());
                beeper.access_denied();
            },
            None => {}
        }
    }
}
