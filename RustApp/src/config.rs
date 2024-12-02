use std::net::IpAddr;

use light_enum::Values;
use serde::{Deserialize, Serialize};
use serde_textual::DisplaySerde;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub connection_mode: ConnectionMode,
    pub ip: Option<IpAddr>,
    pub audio_format: AudioFormat,
    pub channel_count: ChannelCount,
    pub sample_rate: SampleRate,
    // todo: use a device id (https://github.com/RustAudio/cpal/issues/922)
    // i'm not using an index because i'm not sure it will works well with
    // device that could be disconnected sometime.
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Default, PartialEq, Eq, DisplaySerde)]
pub enum ConnectionMode {
    #[default]
    Tcp,
    Udp,
    Adb,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, DisplaySerde, Values)]
pub enum ChannelCount {
    #[default]
    #[serde(alias = "1")]
    Mono,
    #[serde(alias = "2")]
    Stereo,
}

impl ChannelCount {
    pub fn number(&self) -> u16 {
        match self {
            ChannelCount::Mono => 1,
            ChannelCount::Stereo => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, DisplaySerde, Values)]
pub enum AudioFormat {
    #[serde(rename = "i8")]
    I8,
    #[default]
    #[serde(rename = "i16")]
    I16,
    #[serde(rename = "i24")]
    I24,
    #[serde(rename = "i32")]
    I32,
    #[serde(rename = "i48")]
    I48,
    #[serde(rename = "i64")]
    I64,

    #[serde(rename = "u8")]
    U8,
    #[serde(rename = "u16")]
    U16,
    #[serde(rename = "u24")]
    U24,
    #[serde(rename = "u32")]
    U32,
    #[serde(rename = "u48")]
    U48,
    #[serde(rename = "u64")]
    U64,

    #[serde(rename = "f32")]
    F32,
    #[serde(rename = "f64")]
    F64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, DisplaySerde, Values)]
pub enum SampleRate {
    #[serde(rename = "8000")]
    S8000,
    #[serde(rename = "11025")]
    S11025,
    #[default]
    #[serde(rename = "16000")]
    S16000,
    #[serde(rename = "22050")]
    S22050,
    #[serde(rename = "44100")]
    S44100,
    #[serde(rename = "48000")]
    S48000,
    #[serde(rename = "88200")]
    S88200,
    #[serde(rename = "96600")]
    S96600,
    #[serde(rename = "176400")]
    S176400,
    #[serde(rename = "192000")]
    S192000,
    #[serde(rename = "352800")]
    S352800,
    #[serde(rename = "384000")]
    S384000,
}

impl SampleRate {
    pub fn number(&self) -> u32 {
        match self {
            SampleRate::S8000 => 8000,
            SampleRate::S11025 => 11025,
            SampleRate::S16000 => 16000,
            SampleRate::S22050 => 88200,
            SampleRate::S44100 => 88200,
            SampleRate::S48000 => 88200,
            SampleRate::S88200 => 88200,
            SampleRate::S96600 => 96600,
            SampleRate::S176400 => 176400,
            SampleRate::S192000 => 192000,
            SampleRate::S352800 => 352800,
            SampleRate::S384000 => 384000,
        }
    }
}