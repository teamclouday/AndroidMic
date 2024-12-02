use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

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

#[derive(
    Debug, Clone, Serialize, Deserialize, Copy, Default, EnumString, PartialEq, Eq, Display,
)]
pub enum ConnectionMode {
    #[default]
    Tcp,
    Udp,
    Adb,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Display, Default)]
pub enum ChannelCount {
    #[default]
    Mono,
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

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Display, Default)]
pub enum AudioFormat {
    I8,
    #[default]
    I16,
    I24,
    I32,
    I48,
    I64,

    U8,
    U16,
    U24,
    U32,
    U48,
    U64,

    F32,
    F64,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Display, Default)]
pub enum SampleRate {
    S8000,
    S11025,
    #[default]
    S16000,
    S22050,
    S44100,
    S48000,
    S88200,
    S96600,
    S176400,
    S192000,
    S352800,
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
