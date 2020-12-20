use super::socket::ZMQ_CONTEXT;

pub fn get_socket(endpoint: &str) -> Result<zmq::Socket, zmq::Error> {
    info!(
        "Creating new socket for sending commands, endpoint: {}",
        endpoint
    );

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let sock = zmq_ctx.socket(zmq::REQ)?;
    sock.connect(endpoint)?;

    return Ok(sock);
}
