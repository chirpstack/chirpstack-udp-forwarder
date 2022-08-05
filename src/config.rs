use std::fs;

use serde::Deserialize;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize, Default)]
pub struct UdpForwarder {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
    pub metrics_bind: String,
    pub servers: Vec<Server>,
}

#[derive(Deserialize, Default)]
pub struct Server {
    pub server: String,
    pub keepalive_interval_secs: u64,
    pub keepalive_max_failures: u32,
}

#[derive(Deserialize, Default)]
pub struct Concentratord {
    pub event_url: String,
    pub command_url: String,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub udp_forwarder: UdpForwarder,
    pub concentratord: Concentratord,
}

impl Configuration {
    pub fn get(filenames: Vec<String>) -> Result<Configuration, String> {
        let mut content: String = String::new();

        for file_name in &filenames {
            content.push_str(&match fs::read_to_string(file_name) {
                Ok(v) => v,
                Err(err) => return Err(format!("read config file error: {}", err).to_string()),
            });
        }

        let config: Configuration = match toml::from_str(&content) {
            Ok(v) => v,
            Err(err) => return Err(format!("parse config file error: {}", err)),
        };

        return Ok(config);
    }
}
