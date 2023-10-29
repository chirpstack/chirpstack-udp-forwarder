use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::RwLock;
use std::thread;

use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::{Metric, Registry};

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug)]
struct UdpLabels {
    server: String,
    r#type: String,
}

lazy_static! {
    static ref REGISTRY: RwLock<Registry> = RwLock::new(<Registry>::default());

    // UDP sent
    static ref UDP_SENT_COUNT: Family<UdpLabels, Counter> = {
        let counter = Family::<UdpLabels, Counter>::default();
        register("udp_sent_count", "Number of UDP datagrams sent", counter.clone());
        counter
    };
    static ref UDP_SENT_BYTES: Family<UdpLabels, Counter> = {
        let counter = Family::<UdpLabels, Counter>::default();
        register("udp_sent_bytes", "Number of bytes sent over UDP", counter.clone());
        counter
    };


    // UDP received
    static ref UDP_RECEIVED_COUNT: Family<UdpLabels, Counter> = {
        let counter = Family::<UdpLabels, Counter>::default();
        register("udp_received_count", "Number of UDP datagrams received", counter.clone());
        counter
    };
    static ref UDP_RECEIVED_BYTES: Family<UdpLabels, Counter> = {
        let counter = Family::<UdpLabels, Counter>::default();
        register("udp_received_bytes", "Number of bytes received over UDP", counter.clone());
        counter
    };
}

fn register(name: &str, help: &str, metric: impl Metric) {
    let mut registry_w = REGISTRY.write().unwrap();
    registry_w.register(name, help, metric)
}

pub fn start(bind: String) {
    info!("Starting Prometheus metrics server, bind: {}", bind);
    let listener = TcpListener::bind(bind).expect("bind metrics server error");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_request(stream));
            }
            Err(err) => {
                error!("Unable to connect, error: {}", err);
            }
        }
    }
}

pub fn incr_udp_sent_count(server: &str, typ: &str) {
    UDP_SENT_COUNT
        .get_or_create(&UdpLabels {
            server: server.to_string(),
            r#type: typ.to_string(),
        })
        .inc();
}

pub fn incr_udp_sent_bytes(server: &str, typ: &str, count: usize) {
    UDP_SENT_BYTES
        .get_or_create(&UdpLabels {
            server: server.to_string(),
            r#type: typ.to_string(),
        })
        .inc_by(count.try_into().unwrap());
}

pub fn incr_udp_received_count(server: &str, typ: &str) {
    UDP_RECEIVED_COUNT
        .get_or_create(&UdpLabels {
            server: server.to_string(),
            r#type: typ.to_string(),
        })
        .inc();
}

pub fn incr_udp_received_bytes(server: &str, typ: &str, count: usize) {
    UDP_RECEIVED_BYTES
        .get_or_create(&UdpLabels {
            server: server.to_string(),
            r#type: typ.to_string(),
        })
        .inc_by(count.try_into().unwrap());
}

fn handle_request(stream: TcpStream) {
    handle_read(&stream);
    handle_write(stream);
}

fn handle_read(mut stream: &TcpStream) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).unwrap();
}

fn handle_write(mut stream: TcpStream) {
    if let Err(err) =
        stream.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n")
    {
        error!("Write http header error: {}", err);
        return;
    };

    let registry_r = REGISTRY.read().unwrap();
    let mut buffer = String::new();
    if let Err(e) = encode(&mut buffer, &registry_r) {
        error!("Encode Prometheus metrics error: {}", e);
        return;
    }

    if let Err(err) = stream.write(buffer.as_bytes()) {
        error!("Write metrics error: {}", err);
    };
}
