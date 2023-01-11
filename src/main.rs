#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::str::FromStr;
use std::thread;

use clap::Parser;

mod commands;
mod config;
mod events;
mod forwarder;
mod helpers;
mod logging;
mod metrics;
mod signals;
mod socket;
mod structs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let config = config::Configuration::get(&cli.config).expect("read configuration error");
    let log_level =
        log::Level::from_str(&config.udp_forwarder.log_level).expect("parse log_level error");

    logging::setup(
        &"chirpstack-udp-forwarder",
        log_level,
        config.udp_forwarder.log_to_syslog,
    )
    .expect("setup logger error");

    info!(
        "Starting ChirpStack UDP Forwarder (version: {}, docs: {})",
        config::VERSION,
        "https://github.com/chirpstack/chirpstack-udp-forwarder",
    );

    // read gateway id.
    let gateway_id = helpers::get_gateway_id(&config.concentratord.command_url)
        .expect("get gateway_id from concentratord failed, is concentratord running?");

    info!(
        "Received gateway ID from Concentratord, gateway_id: {}",
        hex::encode(&gateway_id)
    );

    // setup threads
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    // servers
    for server in config.udp_forwarder.servers {
        threads.push(thread::spawn({
            let gateway_id = gateway_id.clone();
            let event_url = config.concentratord.event_url.clone();
            let command_url = config.concentratord.command_url.clone();

            move || forwarder::start(&server, event_url, command_url, gateway_id)
        }));
    }

    // metrics
    if config.udp_forwarder.metrics_bind != "" {
        threads.push(thread::spawn({
            let bind = config.udp_forwarder.metrics_bind.clone();
            move || metrics::start(bind)
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}
