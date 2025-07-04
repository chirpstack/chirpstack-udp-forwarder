use anyhow::Result;
use chirpstack_api::{gw, prost::Message};

use super::commands;

pub fn get_gateway_id(command_url: &str) -> Result<Vec<u8>> {
    debug!("Reading gateway id, server: {}", command_url);

    let sock = commands::get_socket(command_url).expect("get client error");

    // Send command.
    let cmd = gw::Command {
        command: Some(gw::command::Command::GetGatewayId(
            gw::GetGatewayIdRequest {},
        )),
    };
    sock.send(cmd.encode_to_vec(), 0).unwrap();

    // set poller so that we can timout after 100ms
    let mut items = [sock.as_poll_item(zmq::POLLIN)];
    zmq::poll(&mut items, 100).unwrap();
    if !items[0].is_readable() {
        return Err(anyhow!("could not read gateway_id"));
    }

    // read 'gateway_id' response
    let b = sock.recv_bytes(0).unwrap();
    let resp = gw::GetGatewayIdResponse::decode(b.as_slice()).unwrap();

    Ok(hex::decode(resp.gateway_id)?)
}
