use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub connection_mode: ConnectionMode,
    pub ip: Option<IpAddr>,
    pub audio_format: AudioFormat,
    pub channel_count: Option<ChannelCount>,
    pub sample_rate: Option<u32>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Copy, Default, EnumString, PartialEq, Eq, Display,
)]
pub enum ConnectionMode {
    #[default]
    #[strum(serialize = "tcp", serialize = "TCP")]
    Tcp,
    #[strum(serialize = "udp", serialize = "UDP")]
    Udp,
    Adb,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Display)]
pub enum ChannelCount {
    #[strum(serialize = "mono", serialize = "MONO", serialize = "1")]
    Mono,
    #[strum(serialize = "stereo", serialize = "STEREO", serialize = "2")]
    Stereo,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Display, Default)]
pub enum AudioFormat {
    #[strum(serialize = "i8")]
    I8,
    #[default]
    #[strum(serialize = "i16")]
    I16,
    #[strum(serialize = "i24")]
    I24,
    #[strum(serialize = "i32")]
    I32,
    #[strum(serialize = "i48")]
    I48,
    #[strum(serialize = "i64")]
    I64,

    #[strum(serialize = "u8")]
    U8,
    #[strum(serialize = "u16")]
    U16,
    #[strum(serialize = "u24")]
    U24,
    #[strum(serialize = "u32")]
    U32,
    #[strum(serialize = "u48")]
    U48,
    #[strum(serialize = "u64")]
    U64,

    #[strum(serialize = "f32")]
    F32,
    #[strum(serialize = "f64")]
    F64,
}
