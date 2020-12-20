use std::time::Duration;

use prost::Message;

use super::socket::ZMQ_CONTEXT;

pub fn get_socket(endpoint: &str) -> Result<zmq::Socket, zmq::Error> {
    info!(
        "Creating new socket for receiving events, endpoint: {}",
        endpoint
    );

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let sock = zmq_ctx.socket(zmq::SUB)?;
    sock.connect(&endpoint).expect("ZMQ connect error");
    sock.set_subscribe("".as_bytes())?;

    return Ok(sock);
}

pub enum Event {
    // Reading event timed out.
    Timeout,

    // Error reading event.
    Error(String),

    // Unknown event.
    Unknown(String, Vec<u8>),

    // Uplink event.
    Uplink(chirpstack_api::gw::UplinkFrame),

    // Stats event.
    Stats(chirpstack_api::gw::GatewayStats),
}

pub struct Reader<'a> {
    sub_sock: &'a zmq::Socket,
    timeout: Duration,
}

impl<'a> Reader<'a> {
    pub fn new(sock: &'a zmq::Socket, timeout: Duration) -> Self {
        Reader {
            sub_sock: sock,
            timeout: timeout,
        }
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

fn handle_message(msg: Vec<Vec<u8>>) -> Result<Event, String> {
    if msg.len() != 2 {
        return Err("event must have two frames".to_string());
    }

    let event = match String::from_utf8(msg[0].clone()) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };

    Ok(match event.as_str() {
        "up" => match parse_up(&msg[1]) {
            Ok(v) => Event::Uplink(v),
            Err(err) => Event::Error(err),
        },
        "stats" => match parse_stats(&msg[1]) {
            Ok(v) => Event::Stats(v),
            Err(err) => Event::Error(err),
        },
        _ => Event::Unknown(event, msg[1].clone()),
    })
}

fn parse_up(msg: &[u8]) -> Result<chirpstack_api::gw::UplinkFrame, String> {
    match chirpstack_api::gw::UplinkFrame::decode(msg) {
        Ok(v) => Ok(v),
        Err(err) => Err(err.to_string()),
    }
}

fn parse_stats(msg: &[u8]) -> Result<chirpstack_api::gw::GatewayStats, String> {
    match chirpstack_api::gw::GatewayStats::decode(msg) {
        Ok(v) => Ok(v),
        Err(err) => Err(err.to_string()),
    }
}
