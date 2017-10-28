#[macro_use] extern crate log;
#[macro_use] extern crate chan;
extern crate clap;
extern crate daemonize;
extern crate chan_signal;
extern crate log4rs;

use std::thread;
use clap::{App, Arg, ArgMatches};
use daemonize::{Daemonize};
use chan_signal::Signal;
use log::LogLevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};

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
        .arg(Arg::with_name("daemonize")
            .short("d")
            .long("daemonize")
            .help("Daemonize instead of running in foreground"))
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

            // TODO: Cleanup

            info!("Shutdown complete. Goodbye!");
        },
        rdone.recv() => {
            info!("Reached exit condition, shutting down");

            // TODO: Cleanup

            info!("Shutdown complete. Goodbye!");
        }
    }
}

fn run(_sdone: chan::Sender<()>, args: ArgMatches) {
    // Should we daemonize or stay in foreground?
    if args.is_present("daemonize") {
        info!("Starting Gatekeeper Door Daemon");
        let daemonize = Daemonize::new()
            .pid_file("./gatekeeper.pid");

        match daemonize.start() {
            Ok(_) => {
                info!("Success, daemonized!");
                std::thread::sleep(std::time::Duration::from_secs(5));
            },
            Err(e) => error!("{}", e),
        }
    } else {
        // Stay in foreground
        info!("Running in foreground");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // _sdone gets dropped which closes the channel and causes `rdone` to unblock
}