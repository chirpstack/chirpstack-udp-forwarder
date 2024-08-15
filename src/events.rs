use std::time::Duration;

use anyhow::Result;
use prost::Message;

use super::socket::ZMQ_CONTEXT;

pub fn get_socket(endpoint: &str) -> Result<zmq::Socket, zmq::Error> {
    info!(
        "Creating new socket for receiving events, endpoint: {}",
        endpoint
    );

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let sock = zmq_ctx.socket(zmq::SUB)?;
    sock.connect(endpoint).expect("ZMQ connect error");
    sock.set_subscribe("".as_bytes())?;

    Ok(sock)
}

pub enum Event {
    // Reading event timed out.
    Timeout,

    // Error reading event.
    Error(String),

    // Unknown event.
    Unknown(String),

    // Uplink event.
    Uplink(Box<chirpstack_api::gw::UplinkFrame>),

    // Stats event.
    Stats(Box<chirpstack_api::gw::GatewayStats>),
}

pub struct Reader<'a> {
    sub_sock: &'a zmq::Socket,
    timeout: Duration,
}

impl<'a> Reader<'a> {
    pub fn new(sub_sock: &'a zmq::Socket, timeout: Duration) -> Self {
        Reader { sub_sock, timeout }
    }
}

impl Iterator for Reader<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        // set poller so that we can timeout
        let mut items = [self.sub_sock.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, self.timeout.as_millis() as i64).unwrap();
        if !items[0].is_readable() {
            return Some(Event::Timeout);
        }

        let msg = self.sub_sock.recv_multipart(0).unwrap();
        match handle_message(msg) {
            Ok(v) => Some(v),
            Err(err) => Some(Event::Error(err.to_string())),
        }
    }
}

fn handle_message(msg: Vec<Vec<u8>>) -> Result<Event> {
    if msg.len() != 2 {
        return Err(anyhow!("Event must have two frames"));
    }

    let event = String::from_utf8(msg[0].clone())?;

    Ok(match event.as_str() {
        "up" => match parse_up(&msg[1]) {
            Ok(v) => Event::Uplink(Box::new(v)),
            Err(err) => Event::Error(err.to_string()),
        },
        "stats" => match parse_stats(&msg[1]) {
            Ok(v) => Event::Stats(Box::new(v)),
            Err(err) => Event::Error(err.to_string()),
        },
        _ => Event::Unknown(event),
    })
}

fn parse_up(msg: &[u8]) -> Result<chirpstack_api::gw::UplinkFrame> {
    Ok(chirpstack_api::gw::UplinkFrame::decode(msg)?)
}

fn parse_stats(msg: &[u8]) -> Result<chirpstack_api::gw::GatewayStats> {
    Ok(chirpstack_api::gw::GatewayStats::decode(msg)?)
}
