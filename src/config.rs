use std::{env, fs};

use anyhow::Result;
use serde::Deserialize;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
#[serde(default)]
pub struct UdpForwarder {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
    pub metrics_bind: String,
    pub servers: Vec<Server>,
}

impl Default for UdpForwarder {
    fn default() -> Self {
        UdpForwarder {
            log_level: "INFO".to_string(),
            log_to_syslog: false,
            metrics_bind: "".to_string(),
            servers: vec![],
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
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

#[derive(Deserialize)]
#[serde(default)]
pub struct Concentratord {
    pub event_url: String,
    pub command_url: String,
}

impl Default for Concentratord {
    fn default() -> Self {
        Concentratord {
            event_url: "ipc:///tmp/concentratord_event".to_string(),
            command_url: "ipc:///tmp/concentratord_command".to_string(),
        }
    }
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

        // Replace environment variables in config.
        for (k, v) in env::vars() {
            content = content.replace(&format!("${}", k), &v);
        }

        let config: Configuration = match toml::from_str(&content) {
            Ok(v) => v,
            Err(err) => return Err(anyhow!("parse config file error: {}", err)),
        };

        Ok(config)
    }
}
