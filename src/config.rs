use std::fs;

use anyhow::Result;
use serde::Deserialize;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize, Default)]
pub struct UdpForwarder {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
    pub metrics_bind: String,
    pub servers: Vec<Server>,
}

#[derive(Deserialize)]
pub struct Server {
    pub server: String,
    pub keepalive_interval_secs: u64,
    pub keepalive_max_failures: u32,
    pub forward_crc_ok: bool,
    pub forward_crc_invalid: bool,
    pub forward_crc_missing: bool,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            server: "127.0.0.1:1700".into(),
            keepalive_interval_secs: 10,
            keepalive_max_failures: 12,
            forward_crc_ok: true,
            forward_crc_invalid: false,
            forward_crc_missing: false,
        }
    }
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
    pub fn get(filenames: &[String]) -> Result<Configuration> {
        let mut content: String = String::new();

        for file_name in filenames {
            content.push_str(&match fs::read_to_string(file_name) {
                Ok(v) => v,
                Err(err) => return Err(anyhow!("read config file error: {}", err)),
            });
        }

        let config: Configuration = match toml::from_str(&content) {
            Ok(v) => v,
            Err(err) => return Err(anyhow!("parse config file error: {}", err)),
        };

        Ok(config)
    }
}
