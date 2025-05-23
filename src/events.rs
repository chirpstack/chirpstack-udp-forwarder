use std::time::Duration;

use anyhow::Result;
use chirpstack_api::{gw, prost::Message};

use super::socket::ZMQ_CONTEXT;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Timeout")]
    Timeout,

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

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
    type Item = Result<gw::Event, Error>;

    fn next(&mut self) -> Option<Result<gw::Event, Error>> {
        // set poller so that we can timeout
        let mut items = [self.sub_sock.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, self.timeout.as_millis() as i64).unwrap();
        if !items[0].is_readable() {
            return Some(Err(Error::Timeout));
        }

        let b = self.sub_sock.recv_bytes(0).unwrap();
        match gw::Event::decode(b.as_slice()).map_err(|e| Error::Anyhow(anyhow::Error::new(e))) {
            Ok(v) => Some(Ok(v)),
            Err(e) => Some(Err(e)),
        }
    }
}
