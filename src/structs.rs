use std::convert::TryInto;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use chirpstack_api::gw;

const PROTOCOL_VERSION: u8 = 0x02;

pub enum CRC {
    OK,
}

impl Serialize for CRC {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CRC::OK => serializer.serialize_i32(1),
        }
    }
}

pub enum Modulation {
    LoRa,
    Fsk,
}

impl Serialize for Modulation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Modulation::LoRa => serializer.serialize_str(&"LORA"),
            Modulation::Fsk => serializer.serialize_str(&"FSK"),
        }
    }
}

impl<'de> Deserialize<'de> for Modulation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "LORA" => Ok(Modulation::LoRa),
            "FSK" => Ok(Modulation::Fsk),
            _ => Err(D::Error::custom("unexpected value"))?,
        }
    }
}

pub enum DataRate {
    LoRa(u32, u32), // SF and BW (kHz)
    FSK(u32),       // bitrate
}

impl Serialize for DataRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DataRate::LoRa(sf, bw) => serializer.serialize_str(&format!("SF{}BW{}", sf, bw / 1000)),
            DataRate::FSK(bitrate) => serializer.serialize_u32(*bitrate),
        }
    }
}

impl<'de> Deserialize<'de> for DataRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::String(v) => {
                let s: Vec<&str> = v.split(char::is_alphabetic).collect();
                if s.len() != 5 {
                    return Err(D::Error::custom("invalid datarate string"));
                }

                let sf: u32 = match s[2].parse() {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(D::Error::custom(format!("parse sf error: {}", err)));
                    }
                };
                let bw: u32 = match s[4].parse() {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(D::Error::custom(format!("parse bw error: {}", err)));
                    }
                };

                return Ok(DataRate::LoRa(sf, bw * 1000));
            }
            Value::Number(v) => {
                // let bitrate = u32::deserialize(deserializer)?;
                let br = v.as_u64().unwrap();
                return Ok(DataRate::FSK(br as u32));
            }
            _ => return Err(D::Error::custom("unexpected type")),
        }
    }
}

#[derive(Clone, Copy)]
pub enum CodeRate {
    Undefined,
    LoRa4_5,
    LoRa4_6,
    LoRa4_7,
    LoRa4_8,
}

impl Serialize for CodeRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CodeRate::LoRa4_5 => serializer.serialize_str(&"4/5"),
            CodeRate::LoRa4_6 => serializer.serialize_str(&"4/6"),
            CodeRate::LoRa4_7 => serializer.serialize_str(&"4/7"),
            CodeRate::LoRa4_8 => serializer.serialize_str(&"4/8"),
            _ => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for CodeRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "4/5" => Ok(CodeRate::LoRa4_5),
            "4/6" => Ok(CodeRate::LoRa4_6),
            "4/7" => Ok(CodeRate::LoRa4_7),
            "4/8" => Ok(CodeRate::LoRa4_8),
            _ => Ok(CodeRate::Undefined),
        }
    }
}

pub struct PushData {
    pub random_token: u16,
    pub gateway_id: [u8; 8],
    pub payload: PushDataPayload,
}

impl PushData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();

        b.push(PROTOCOL_VERSION);
        b.append(&mut self.random_token.to_be_bytes().to_vec());
        b.push(0x00);
        b.append(&mut self.gateway_id.to_vec());

        let mut j = serde_json::to_vec(&self.payload).unwrap();
        b.append(&mut j);

        return b;
    }
}

#[derive(Serialize)]
pub struct PushDataPayload {
    pub rxpk: Vec<RXPK>,
    pub stat: Option<Stat>,
}

#[derive(Serialize)]
pub struct RXPK {
    /// UTC time of pkt RX, us precision, ISO 8601 'compact' format
    #[serde(with = "compact_time_format")]
    pub time: DateTime<Utc>,
    /// GPS time of pkt RX, number of milliseconds since 06.Jan.1980
    pub tmms: Option<u64>,
    /// Internal timestamp of "RX finished" event (32b unsigned)
    pub tmst: u32,
    /// RX central frequency in MHz (unsigned float, Hz precision)
    pub freq: f64,
    /// Concentrator "IF" channel used for RX (unsigned integer)
    pub chan: u32,
    /// Concentrator "RF chain" used for RX (unsigned integer)
    pub rfch: u32,
    /// CRC status: 1 = OK, -1 = fail, 0 = no CRC
    pub stat: CRC,
    /// Modulation identifier "LORA" or "FSK"
    pub modu: Modulation,
    /// LoRa datarate identifier (eg. SF12BW500)}
    pub datr: DataRate,
    /// LoRa coding rate.
    pub codr: Option<CodeRate>,
    /// RSSI in dBm (signed integer, 1 dB precision).
    pub rssi: i32,
    /// Lora SNR ratio in dB (signed float, 0.1 dB precision).
    pub lsnr: Option<f32>,
    /// RF packet payload size in bytes (unsigned integer).
    pub size: u8,
    /// Base64 encoded RF packet payload, padded.
    pub data: String,
}

impl RXPK {
    pub fn from_proto(up: &chirpstack_api::gw::UplinkFrame) -> Result<Self, String> {
        let rx_info = match &up.rx_info {
            Some(v) => v,
            None => {
                return Err("rx_info must not be None".to_string());
            }
        };

        let tx_info = match &up.tx_info {
            Some(v) => v,
            None => {
                return Err("tx_info must not be None".to_string());
            }
        };

        Ok(RXPK {
            time: DateTime::from(match &rx_info.time {
                Some(v) => match v.clone().try_into() {
                    Ok(v) => v,
                    Err(_) => Utc::now(),
                },
                None => Utc::now(),
            }),
            tmms: rx_info
                .time_since_gps_epoch
                .as_ref()
                .map(|v| (v.seconds * 1000) as u64 + (v.nanos / 1000000) as u64),
            tmst: {
                let mut bytes: [u8; 4] = [0; 4];
                bytes.copy_from_slice(&rx_info.context);
                u32::from_be_bytes(bytes)
            },
            freq: tx_info.frequency as f64 / 1000000.0,
            chan: rx_info.channel,
            rfch: rx_info.rf_chain,
            stat: CRC::OK,
            modu: match &tx_info.modulation {
                Some(v) => match &v.parameters {
                    Some(v) => match &v {
                        gw::modulation::Parameters::Lora(_) => Modulation::LoRa,
                        gw::modulation::Parameters::Fsk(_) => Modulation::Fsk,
                        gw::modulation::Parameters::LrFhss(_) => {
                            return Err("unsupported modulation".to_string());
                        }
                    },
                    None => {
                        return Err("parameters must not be None".to_string());
                    }
                },
                None => {
                    return Err("modulation_info must not be None".to_string());
                }
            },
            datr: match &tx_info.modulation {
                Some(v) => match &v.parameters {
                    Some(v) => match &v {
                        gw::modulation::Parameters::Lora(v) => {
                            DataRate::LoRa(v.spreading_factor, v.bandwidth)
                        }
                        gw::modulation::Parameters::Fsk(v) => DataRate::FSK(v.datarate),
                        gw::modulation::Parameters::LrFhss(_) => {
                            return Err("unsupported modulation".to_string());
                        }
                    },
                    None => {
                        return Err("parameters must not be None".to_string());
                    }
                },
                None => {
                    return Err("modulation_info must not be None".to_string());
                }
            },
            codr: match &tx_info.modulation {
                Some(v) => match &v.parameters {
                    Some(v) => match &v {
                        gw::modulation::Parameters::Lora(v) => Some(match v.code_rate() {
                            gw::CodeRate::Cr45 => CodeRate::LoRa4_5,
                            gw::CodeRate::Cr46 => CodeRate::LoRa4_6,
                            gw::CodeRate::Cr47 => CodeRate::LoRa4_7,
                            gw::CodeRate::Cr48 => CodeRate::LoRa4_8,
                            _ => CodeRate::Undefined,
                        }),
                        _ => None,
                    },
                    None => None,
                },
                None => None,
            },
            rssi: rx_info.rssi,
            lsnr: match &tx_info.modulation {
                Some(v) => match &v.parameters {
                    Some(v) => match &v {
                        gw::modulation::Parameters::Lora(_) => Some(rx_info.snr as f32),
                        _ => None,
                    },
                    None => None,
                },
                None => None,
            },
            size: up.phy_payload.len() as u8,
            data: base64::encode(up.phy_payload.clone()),
        })
    }
}

#[derive(Serialize)]
pub struct Stat {
    /// UTC 'system' time of the gateway, ISO 8601 'expanded' format.
    #[serde(with = "expanded_time_format")]
    pub time: DateTime<Utc>,
    /// GPS latitude of the gateway in degree (float, N is +).
    pub lati: f64,
    /// GPS latitude of the gateway in degree (float, E is +).
    pub long: f64,
    /// GPS altitude of the gateway in meter RX (integer).
    pub alti: u32,
    /// Number of radio packets received (unsigned integer).
    pub rxnb: u32,
    /// Number of radio packets received with a valid PHY CRC.
    pub rxok: u32,
    /// Number of radio packets forwarded (unsigned integer).
    pub rxfw: u32,
    /// Percentage of upstream datagrams that were acknowledged.
    pub ackr: f32,
    /// Number of downlink datagrams received (unsigned integer).
    pub dwnb: u32,
    /// Number of packets emitted (unsigned integer).
    pub txnb: u32,
}

impl Stat {
    pub fn from_proto(stats: &chirpstack_api::gw::GatewayStats) -> Result<Self, String> {
        Ok(Stat {
            time: match &stats.time {
                Some(v) => match v.clone().try_into() {
                    Ok(v) => v,
                    Err(_) => Utc::now(),
                },
                None => Utc::now(),
            },
            lati: match &stats.location {
                Some(v) => v.latitude,
                None => 0.0,
            },
            long: match &stats.location {
                Some(v) => v.longitude,
                None => 0.0,
            },
            alti: match &stats.location {
                Some(v) => v.altitude as u32,
                None => 0,
            },
            rxnb: stats.rx_packets_received,
            rxok: stats.rx_packets_received_ok,
            rxfw: 0,
            ackr: 0.0,
            dwnb: stats.tx_packets_received,
            txnb: stats.tx_packets_emitted,
        })
    }
}

pub struct PushAck {
    pub random_token: u16,
}

impl PushAck {
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() != 4 {
            return Err(format!("expected 4 bytes, got: {}", b.len()).to_string());
        }

        if b[0] != PROTOCOL_VERSION {
            return Err(format!(
                "expected protocol version: {}, got: {}",
                PROTOCOL_VERSION, b[0]
            )
            .to_string());
        }

        if b[3] != 0x01 {
            return Err(format!("invalid identifier: {}", b[3]).to_string());
        }

        let mut rt: [u8; 2] = [0; 2];
        rt.copy_from_slice(&b[1..3]);

        Ok(PushAck {
            random_token: u16::from_be_bytes(rt),
        })
    }
}

pub struct PullData {
    pub random_token: u16,
    pub gateway_id: [u8; 8],
}

impl PullData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::with_capacity(12);
        b.push(PROTOCOL_VERSION);
        b.append(&mut self.random_token.to_be_bytes().to_vec());
        b.push(0x02);
        b.append(&mut self.gateway_id.to_vec());

        return b;
    }
}

pub struct PullAck {
    pub random_token: u16,
}

impl PullAck {
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() != 4 {
            return Err(format!("expected 4 bytes, got: {}", b.len()).to_string());
        }

        if b[0] != PROTOCOL_VERSION {
            return Err(format!(
                "expected protocol version: {}, got: {}",
                PROTOCOL_VERSION, b[0]
            )
            .to_string());
        }

        if b[3] != 0x04 {
            return Err(format!("invalid identifier: {}", b[3]).to_string());
        }

        let mut rt: [u8; 2] = [0; 2];
        rt.copy_from_slice(&b[1..3]);

        Ok(PullAck {
            random_token: u16::from_be_bytes(rt),
        })
    }
}

pub struct PullResp {
    pub random_token: u16,
    pub payload: PullRespPayload,
}

impl PullResp {
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() < 5 {
            return Err(format!("expected at least 5 bytes, got: {}", b.len()).to_string());
        }

        if b[0] != PROTOCOL_VERSION {
            return Err(format!(
                "expected protocol version: {}, got: {}",
                PROTOCOL_VERSION, b[0]
            )
            .to_string());
        }

        if b[3] != 0x03 {
            return Err(format!("invalid identifier: {}", b[3]).to_string());
        }

        let mut rt: [u8; 2] = [0; 2];
        rt.copy_from_slice(&b[1..3]);

        let pl: PullRespPayload = match serde_json::from_slice(&b[4..]) {
            Ok(v) => v,
            Err(err) => {
                return Err(err.to_string());
            }
        };

        Ok(PullResp {
            random_token: u16::from_be_bytes(rt),
            payload: pl,
        })
    }
}

#[derive(Deserialize)]
pub struct PullRespPayload {
    pub txpk: TXPK,
}

#[derive(Deserialize)]
pub struct TXPK {
    /// Send packet immediately (will ignore tmst & time).
    pub imme: Option<bool>,
    /// Send packet on a certain timestamp value (will ignore time).
    pub tmst: Option<u32>,
    /// Send packet at a certain GPS time (GPS synchronization required).
    pub tmms: Option<u64>,
    /// TX central frequency in MHz (unsigned float, Hz precision).
    pub freq: f64,
    /// Concentrator "RF chain" used for TX (unsigned integer).
    pub rfch: u8,
    /// TX output power in dBm (unsigned integer, dBm precision).
    pub powe: u8,
    /// Modulation identifier "LORA" or "FSK".
    pub modu: Modulation,
    /// LoRa datarate identifier (eg. SF12BW500).
    pub datr: DataRate,
    /// LoRa ECC coding rate identifier.
    pub codr: Option<CodeRate>,
    /// FSK frequency deviation (unsigned integer, in Hz) .
    pub fdev: Option<u32>,
    /// Lora modulation polarization inversion.
    pub ipol: Option<bool>,
    /// RF preamble size (unsigned integer).
    pub prea: Option<u8>,
    /// RF packet payload size in bytes (unsigned integer).
    pub size: u8,
    /// Base64 encoded RF packet payload, padding optional.
    pub data: String,
    /// If true, disable the CRC of the physical layer (optional).
    pub ncrc: Option<bool>,
}

impl TXPK {
    pub fn to_proto(
        &self,
        downlink_id: u32,
        gateway_id: Vec<u8>,
    ) -> Result<chirpstack_api::gw::DownlinkFrame, String> {
        let tx_info = chirpstack_api::gw::DownlinkTxInfo {
            frequency: (self.freq * 1_000_000.0) as u32,
            power: self.powe as i32,
            modulation: Some(gw::Modulation {
                parameters: Some(match self.modu {
                    Modulation::LoRa => match self.datr {
                        DataRate::LoRa(sf, bw) => {
                            gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                                bandwidth: bw,
                                spreading_factor: sf,
                                code_rate: match self.codr {
                                    Some(CodeRate::LoRa4_5) => gw::CodeRate::Cr45,
                                    Some(CodeRate::LoRa4_6) => gw::CodeRate::Cr46,
                                    Some(CodeRate::LoRa4_7) => gw::CodeRate::Cr47,
                                    Some(CodeRate::LoRa4_8) => gw::CodeRate::Cr48,
                                    Some(CodeRate::Undefined) | None => gw::CodeRate::CrUndefined,
                                }
                                .into(),
                                polarization_inversion: self.ipol.unwrap_or(true),
                                ..Default::default()
                            })
                        }
                        _ => {
                            return Err("LoRa DataRate expected".to_string());
                        }
                    },
                    Modulation::Fsk => match self.datr {
                        DataRate::FSK(v) => {
                            gw::modulation::Parameters::Fsk(gw::FskModulationInfo {
                                datarate: v,
                                frequency_deviation: self.fdev.unwrap_or(0),
                            })
                        }
                        _ => {
                            return Err("FSK DataRate expected".to_string());
                        }
                    },
                }),
            }),
            board: 0,
            antenna: 0,
            timing: Some(gw::Timing {
                parameters: Some(if self.imme.unwrap_or(false) {
                    gw::timing::Parameters::Immediately(gw::ImmediatelyTimingInfo {})
                } else if let Some(_) = self.tmst {
                    gw::timing::Parameters::Delay(gw::DelayTimingInfo {
                        delay: Some(pbjson_types::Duration {
                            // This is correct! The delay is already added to the tmst which is
                            // used to set the context.
                            seconds: 0,
                            nanos: 0,
                        }),
                    })
                } else if let Some(v) = self.tmms {
                    gw::timing::Parameters::GpsEpoch(gw::GpsEpochTimingInfo {
                        time_since_gps_epoch: Some(pbjson_types::Duration::from(
                            Duration::from_millis(v),
                        )),
                    })
                } else {
                    return Err("no timing information found".to_string());
                }),
            }),
            context: self
                .tmst
                .map(|v| v.to_be_bytes().to_vec())
                .unwrap_or(vec![]),
        };

        return Ok(chirpstack_api::gw::DownlinkFrame {
            downlink_id: downlink_id,
            gateway_id: hex::encode(gateway_id),
            items: vec![chirpstack_api::gw::DownlinkFrameItem {
                tx_info: Some(tx_info),
                phy_payload: match base64::decode(&self.data) {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(format!("base64 decode payload error: {}", err).to_string());
                    }
                },
                ..Default::default()
            }],
            ..Default::default()
        });
    }
}

pub struct TxAck {
    pub random_token: u16,
    pub gateway_id: [u8; 8],
    pub payload: TxAckPayload,
}

impl TxAck {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();

        b.push(PROTOCOL_VERSION);
        b.append(&mut self.random_token.to_be_bytes().to_vec());
        b.push(0x05);
        b.append(&mut self.gateway_id.to_vec());

        let mut j = serde_json::to_vec(&self.payload).unwrap();
        b.append(&mut j);

        return b;
    }
}

#[derive(Serialize)]
pub struct TxAckPayload {
    pub txpk_ack: TxAckPayloadError,
}

#[derive(Serialize)]
pub struct TxAckPayloadError {
    pub error: String,
}

// see: https://serde.rs/custom-date-format.html
mod expanded_time_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S %Z";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}

mod compact_time_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    const FORMAT: &'static str = "%+";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{DateTime, Utc};
    use std::str;
    use std::time::{Duration, SystemTime};

    use chirpstack_api::{common, gw};

    #[test]
    fn test_push_data_rxpk_lora() {
        let now: DateTime<Utc> = DateTime::from(SystemTime::UNIX_EPOCH);

        let rx_info = gw::UplinkRxInfo {
            gateway_id: "0102030405060708".into(),
            time: Some(pbjson_types::Timestamp::from(now)),
            time_since_gps_epoch: Some(pbjson_types::Duration::from(Duration::from_secs(1))),
            rssi: -160,
            snr: 5.5,
            board: 2,
            channel: 1,
            rf_chain: 1,
            antenna: 3,
            context: vec![1, 2, 3, 4],
            ..Default::default()
        };

        let tx_info = gw::UplinkTxInfo {
            frequency: 868300000,
            modulation: Some(gw::Modulation {
                parameters: Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                    bandwidth: 125000,
                    spreading_factor: 12,
                    code_rate: gw::CodeRate::Cr45.into(),
                    polarization_inversion: true,
                    ..Default::default()
                })),
            }),
        };

        let uf = gw::UplinkFrame {
            rx_info: Some(rx_info),
            tx_info: Some(tx_info),
            phy_payload: vec![1, 2, 3],
            ..Default::default()
        };

        let rxpk = RXPK::from_proto(&uf).unwrap();
        let pd = PushData {
            random_token: 123,
            gateway_id: [1, 2, 3, 4, 5, 6, 7, 8],
            payload: PushDataPayload {
                rxpk: vec![rxpk],
                stat: None,
            },
        };

        let b = pd.to_bytes();
        assert_eq!(
            b[0..12].to_vec(),
            vec![2, 0, 123, 0, 1, 2, 3, 4, 5, 6, 7, 8]
        );

        assert_eq!(
            str::from_utf8(&b[12..]).unwrap(),
            r#"{"rxpk":[{"time":"1970-01-01T00:00:00+00:00","tmms":1000,"tmst":16909060,"freq":868.3,"chan":1,"rfch":1,"stat":1,"modu":"LORA","datr":"SF12BW125","codr":"4/5","rssi":-160,"lsnr":5.5,"size":3,"data":"AQID"}],"stat":null}"#
        );
    }

    #[test]
    fn test_push_data_rxpk_fsk() {
        let now: DateTime<Utc> = DateTime::from(SystemTime::UNIX_EPOCH);

        let rx_info = gw::UplinkRxInfo {
            gateway_id: "0102030405060708".into(),
            time: Some(pbjson_types::Timestamp::from(now)),
            time_since_gps_epoch: Some(pbjson_types::Duration::from(Duration::from_secs(1))),
            rssi: -160,
            channel: 1,
            rf_chain: 2,
            board: 3,
            antenna: 4,
            context: vec![1, 2, 3, 4],
            ..Default::default()
        };

        let tx_info = gw::UplinkTxInfo {
            frequency: 868300000,
            modulation: Some(gw::Modulation {
                parameters: Some(gw::modulation::Parameters::Fsk(gw::FskModulationInfo {
                    datarate: 50000,
                    ..Default::default()
                })),
            }),
        };

        let uf = gw::UplinkFrame {
            rx_info: Some(rx_info),
            tx_info: Some(tx_info),
            phy_payload: vec![1, 2, 3],
            ..Default::default()
        };

        let rxpk = RXPK::from_proto(&uf).unwrap();
        let pd = PushData {
            random_token: 123,
            gateway_id: [1, 2, 3, 4, 5, 6, 7, 8],
            payload: PushDataPayload {
                rxpk: vec![rxpk],
                stat: None,
            },
        };

        let b = pd.to_bytes();
        assert_eq!(
            b[0..12].to_vec(),
            vec![2, 0, 123, 0, 1, 2, 3, 4, 5, 6, 7, 8]
        );

        assert_eq!(
            str::from_utf8(&b[12..]).unwrap(),
            r#"{"rxpk":[{"time":"1970-01-01T00:00:00+00:00","tmms":1000,"tmst":16909060,"freq":868.3,"chan":1,"rfch":2,"stat":1,"modu":"FSK","datr":50000,"codr":null,"rssi":-160,"lsnr":null,"size":3,"data":"AQID"}],"stat":null}"#
        );
    }

    #[test]
    fn test_push_data_stat() {
        let now: DateTime<Utc> = DateTime::from(SystemTime::UNIX_EPOCH);

        let gs = gw::GatewayStats {
            gateway_id: "0102030405060708".into(),
            time: Some(pbjson_types::Timestamp::from(now)),
            location: Some(common::Location {
                latitude: 1.123,
                longitude: 2.123,
                altitude: 3.123,
                ..Default::default()
            }),
            rx_packets_received: 10,
            rx_packets_received_ok: 5,
            tx_packets_received: 14,
            tx_packets_emitted: 7,
            ..Default::default()
        };

        let stat = Stat::from_proto(&gs).unwrap();
        let pd = PushData {
            random_token: 123,
            gateway_id: [1, 2, 3, 4, 5, 6, 7, 8],
            payload: PushDataPayload {
                rxpk: vec![],
                stat: Some(stat),
            },
        };

        let b = pd.to_bytes();
        assert_eq!(
            b[0..12].to_vec(),
            vec![2, 0, 123, 0, 1, 2, 3, 4, 5, 6, 7, 8]
        );

        assert_eq!(
            str::from_utf8(&b[12..]).unwrap(),
            r#"{"rxpk":[],"stat":{"time":"1970-01-01 00:00:00 UTC","lati":1.123,"long":2.123,"alti":3,"rxnb":10,"rxok":5,"rxfw":0,"ackr":0.0,"dwnb":14,"txnb":7}}"#
        );
    }

    #[test]
    fn test_push_ack() {
        let b: [u8; 4] = [2, 0, 123, 1];

        let push_ack = PushAck::from_bytes(&b).unwrap();
        assert_eq!(push_ack.random_token, 123);
    }

    #[test]
    fn test_pull_data() {
        let pull_data = PullData {
            random_token: 123,
            gateway_id: [1, 2, 3, 4, 5, 6, 7, 8],
        };

        let b = pull_data.to_bytes();
        assert_eq!(b, [2, 0, 123, 2, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_pull_ack() {
        let b: [u8; 4] = [2, 0, 123, 4];

        let pull_ack = PullAck::from_bytes(&b).unwrap();
        assert_eq!(pull_ack.random_token, 123);
    }

    #[test]
    fn test_pull_resp_lora_immediately() {
        let txpk = r#"{"txpk":{
            "imme":true,
            "freq":864.123456,
            "rfch":0,
            "powe":14,
            "modu":"LORA",
            "datr":"SF11BW125",
            "codr":"4/6",
            "ipol":false,
            "size":32,
            "data":"H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s="}}"#;
        let mut txpk = txpk.as_bytes().to_vec();

        let mut b: Vec<u8> = vec![2, 0, 123, 3];
        b.append(&mut txpk);

        let pull_resp = PullResp::from_bytes(&b).unwrap();

        assert_eq!(pull_resp.random_token, 123);

        let downlink_frame = pull_resp
            .payload
            .txpk
            .to_proto(0, vec![1, 2, 3, 4, 5, 6, 7, 8])
            .unwrap();

        let tx_info = gw::DownlinkTxInfo {
            frequency: 864123456,
            power: 14,
            board: 0,
            antenna: 0,
            context: vec![],
            timing: Some(gw::Timing {
                parameters: Some(gw::timing::Parameters::Immediately(
                    gw::ImmediatelyTimingInfo {},
                )),
            }),
            modulation: Some(gw::Modulation {
                parameters: Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                    bandwidth: 125000,
                    spreading_factor: 11,
                    code_rate: gw::CodeRate::Cr46.into(),
                    polarization_inversion: false,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        assert_eq!(
            downlink_frame,
            gw::DownlinkFrame {
                downlink_id: 0,
                gateway_id: "0102030405060708".into(),
                items: vec![gw::DownlinkFrameItem {
                    phy_payload: base64::decode("H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s=")
                        .unwrap(),
                    tx_info: Some(tx_info),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_pull_resp_lora_delay() {
        let txpk = r#"{"txpk":{
            "freq":864.123456,
            "rfch":0,
            "powe":14,
            "modu":"LORA",
            "datr":"SF11BW125",
            "codr":"4/5",
            "ipol":false,
            "size":32,
            "tmst": 5000000,
            "data":"H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s="}}"#;
        let mut txpk = txpk.as_bytes().to_vec();

        let mut b: Vec<u8> = vec![2, 0, 123, 3];
        b.append(&mut txpk);

        let pull_resp = PullResp::from_bytes(&b).unwrap();

        assert_eq!(pull_resp.random_token, 123);

        let downlink_frame = pull_resp
            .payload
            .txpk
            .to_proto(0, vec![1, 2, 3, 4, 5, 6, 7, 8])
            .unwrap();

        let tx_info = gw::DownlinkTxInfo {
            frequency: 864123456,
            power: 14,
            board: 0,
            antenna: 0,
            context: vec![0, 76, 75, 64], // == 5000000
            timing: Some(gw::Timing {
                parameters: Some(gw::timing::Parameters::Delay(gw::DelayTimingInfo {
                    delay: Some(pbjson_types::Duration::from(Duration::from_secs(0))),
                })),
            }),
            modulation: Some(gw::Modulation {
                parameters: Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                    bandwidth: 125000,
                    spreading_factor: 11,
                    code_rate: gw::CodeRate::Cr45.into(),
                    polarization_inversion: false,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        assert_eq!(
            downlink_frame,
            gw::DownlinkFrame {
                downlink_id: 0,
                gateway_id: "0102030405060708".into(),
                items: vec![gw::DownlinkFrameItem {
                    phy_payload: base64::decode("H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s=")
                        .unwrap(),
                    tx_info: Some(tx_info),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_pull_resp_lora_gps() {
        let txpk = r#"{"txpk":{
            "freq":864.123456,
            "rfch":0,
            "powe":14,
            "modu":"LORA",
            "datr":"SF11BW125",
            "codr":"4/5",
            "ipol":false,
            "size":32,
            "tmms": 5000000,
            "data":"H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s="}}"#;
        let mut txpk = txpk.as_bytes().to_vec();

        let mut b: Vec<u8> = vec![2, 0, 123, 3];
        b.append(&mut txpk);

        let pull_resp = PullResp::from_bytes(&b).unwrap();

        assert_eq!(pull_resp.random_token, 123);

        let downlink_frame = pull_resp
            .payload
            .txpk
            .to_proto(0, vec![1, 2, 3, 4, 5, 6, 7, 8])
            .unwrap();

        let tx_info = gw::DownlinkTxInfo {
            frequency: 864123456,
            power: 14,
            board: 0,
            antenna: 0,
            context: vec![],
            timing: Some(gw::Timing {
                parameters: Some(gw::timing::Parameters::GpsEpoch(gw::GpsEpochTimingInfo {
                    time_since_gps_epoch: Some(pbjson_types::Duration::from(Duration::from_secs(
                        5000,
                    ))),
                })),
            }),
            modulation: Some(gw::Modulation {
                parameters: Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                    bandwidth: 125000,
                    spreading_factor: 11,
                    code_rate: gw::CodeRate::Cr45.into(),
                    polarization_inversion: false,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        assert_eq!(
            downlink_frame,
            gw::DownlinkFrame {
                downlink_id: 0,
                gateway_id: "0102030405060708".into(),
                items: vec![gw::DownlinkFrameItem {
                    phy_payload: base64::decode("H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s=")
                        .unwrap(),
                    tx_info: Some(tx_info),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_pull_resp_fsk_delay() {
        let txpk = r#"{"txpk":{
            "freq":861.3,
            "rfch":0,
            "powe":12,
            "modu":"FSK",
            "datr":50000,
            "fdev":3000,
            "size":32,
            "tmst": 5000000,
            "data":"H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s="}}"#;
        let mut txpk = txpk.as_bytes().to_vec();

        let mut b: Vec<u8> = vec![2, 0, 123, 3];
        b.append(&mut txpk);

        let pull_resp = PullResp::from_bytes(&b).unwrap();

        assert_eq!(pull_resp.random_token, 123);

        let downlink_frame = pull_resp
            .payload
            .txpk
            .to_proto(0, vec![1, 2, 3, 4, 5, 6, 7, 8])
            .unwrap();

        let tx_info = gw::DownlinkTxInfo {
            frequency: 861300000,
            power: 12,
            board: 0,
            antenna: 0,
            context: vec![0, 76, 75, 64], // == 5000000
            timing: Some(gw::Timing {
                parameters: Some(gw::timing::Parameters::Delay(gw::DelayTimingInfo {
                    delay: Some(pbjson_types::Duration::from(Duration::from_secs(0))),
                })),
            }),
            modulation: Some(gw::Modulation {
                parameters: Some(gw::modulation::Parameters::Fsk(gw::FskModulationInfo {
                    frequency_deviation: 3000,
                    datarate: 50000,
                })),
            }),
            ..Default::default()
        };

        assert_eq!(
            downlink_frame,
            gw::DownlinkFrame {
                downlink_id: 0,
                gateway_id: "0102030405060708".into(),
                items: vec![gw::DownlinkFrameItem {
                    phy_payload: base64::decode("H3P3N2i9qc4yt7rK7ldqoeCVJGBybzPY5h1Dd7P7p8s=")
                        .unwrap(),
                    tx_info: Some(tx_info),
                    ..Default::default()
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_tx_ack() {
        let tx_ack = TxAck {
            random_token: 123,
            gateway_id: [1, 2, 3, 4, 5, 6, 7, 8],
            payload: TxAckPayload {
                txpk_ack: TxAckPayloadError {
                    error: "TOO_LATE".to_string(),
                },
            },
        };

        let b = tx_ack.to_bytes();
        assert_eq!(
            b[0..12].to_vec(),
            vec![2, 0, 123, 5, 1, 2, 3, 4, 5, 6, 7, 8],
        );

        assert_eq!(
            str::from_utf8(&b[12..]).unwrap(),
            r#"{"txpk_ack":{"error":"TOO_LATE"}}"#,
        );
    }
}
