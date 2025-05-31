use std::net::IpAddr;

use clap::Parser;
use light_enum::Values;
use serde::{Deserialize, Serialize};

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
    pub start_at_login: bool,
    pub auto_connect: bool,
    pub denoise: bool,
}

#[derive(Parser, Debug)]
#[clap(author = "wiiznokes", version, about = "AndroidMic", long_about = None)]
pub struct Args {
    #[arg(short, long, help = "example: -i 192.168.1.79")]
    pub ip: Option<IpAddr>,

    #[arg(
        short = 'm',
        long = "mode",
        id = "connection mode",
        help = "UDP or TCP"
    )]
    pub connection_mode: Option<ConnectionMode>,

    #[arg(short = 'd', long = "device", id = "output device")]
    pub output_device: Option<String>,

    #[arg(
        short = 'f',
        long = "format",
        id = "audio format",
        help = "i16, f32, ..."
    )]
    pub audio_format: Option<AudioFormat>,

    #[arg(short = 'c', long = "channel", id = "channel count", help = "1 or 2")]
    pub channel_count: Option<ChannelCount>,

    #[arg(
        short = 's',
        long = "sample",
        id = "sample rate",
        help = "16000, 44100, ..."
    )]
    pub sample_rate: Option<SampleRate>,

    #[arg(
        long = "info",
        id = "supported audio config",
        help = "show supported audio config",
        default_value_t = false
    )]
    pub show_supported_audio_config: bool,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
pub enum ConnectionMode {
    #[default]
    #[strum(serialize = "tcp", serialize = "TCP")]
    Tcp,
    #[strum(serialize = "udp", serialize = "UDP")]
    Udp,
    #[strum(serialize = "adb", serialize = "ADB")]
    Adb,
    #[strum(serialize = "usb", serialize = "USB")]
    Usb,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    Values,
    strum::Display,
    strum::EnumString,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
pub enum ChannelCount {
    #[default]
    #[strum(serialize = "mono", serialize = "MONO", serialize = "1")]
    Mono,
    #[strum(serialize = "stereo", serialize = "STEREO", serialize = "2")]
    Stereo,
}

impl ChannelCount {
    pub fn to_number(&self) -> u16 {
        match self {
            ChannelCount::Mono => 1,
            ChannelCount::Stereo => 2,
        }
    }

    pub fn from_number(value: u16) -> Option<Self> {
        match value {
            1 => Some(ChannelCount::Mono),
            2 => Some(ChannelCount::Stereo),
            _ => None,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    Values,
    strum::Display,
    strum::EnumString,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
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

impl AudioFormat {
    pub fn sample_size(&self) -> usize {
        match self {
            AudioFormat::I8 => 1,
            AudioFormat::I16 => 2,
            AudioFormat::I24 => 3,
            AudioFormat::I32 => 4,
            AudioFormat::I48 => 6,
            AudioFormat::I64 => 8,
            AudioFormat::U8 => 1,
            AudioFormat::U16 => 2,
            AudioFormat::U24 => 3,
            AudioFormat::U32 => 4,
            AudioFormat::U48 => 6,
            AudioFormat::U64 => 8,
            AudioFormat::F32 => 4,
            AudioFormat::F64 => 8,
        }
    }

    pub fn from_android_format(format: u32) -> Option<Self> {
        match format {
            3 => Some(AudioFormat::U8),
            2 => Some(AudioFormat::I16),
            21 => Some(AudioFormat::I24),
            22 => Some(AudioFormat::I32),
            4 => Some(AudioFormat::F32),
            _ => None,
        }
    }

    pub fn from_cpal_format(format: cpal::SampleFormat) -> Option<Self> {
        match format {
            cpal::SampleFormat::I8 => Some(AudioFormat::I8),
            cpal::SampleFormat::U8 => Some(AudioFormat::U8),
            cpal::SampleFormat::I16 => Some(AudioFormat::I16),
            cpal::SampleFormat::U16 => Some(AudioFormat::U16),
            cpal::SampleFormat::I32 => Some(AudioFormat::I32),
            cpal::SampleFormat::F32 => Some(AudioFormat::I24),
            cpal::SampleFormat::I64 => Some(AudioFormat::I64),
            cpal::SampleFormat::U64 => Some(AudioFormat::U64),
            _ => None,
        }
    }
}

impl PartialEq<cpal::SampleFormat> for AudioFormat {
    fn eq(&self, other: &cpal::SampleFormat) -> bool {
        match self {
            AudioFormat::I8 => *other == cpal::SampleFormat::I8,
            AudioFormat::U8 => *other == cpal::SampleFormat::U8,
            AudioFormat::I16 => *other == cpal::SampleFormat::I16,
            AudioFormat::U16 => *other == cpal::SampleFormat::U16,
            AudioFormat::I32 => *other == cpal::SampleFormat::I32,
            AudioFormat::I24 => *other == cpal::SampleFormat::F32,
            AudioFormat::I48 => *other == cpal::SampleFormat::I64,
            AudioFormat::I64 => *other == cpal::SampleFormat::I64,
            AudioFormat::U24 => *other == cpal::SampleFormat::F32,
            AudioFormat::U32 => *other == cpal::SampleFormat::U32,
            AudioFormat::U48 => *other == cpal::SampleFormat::U64,
            AudioFormat::U64 => *other == cpal::SampleFormat::U64,
            AudioFormat::F32 => *other == cpal::SampleFormat::F32,
            AudioFormat::F64 => *other == cpal::SampleFormat::F64,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    Values,
    strum::Display,
    strum::EnumString,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
pub enum SampleRate {
    #[strum(serialize = "8000")]
    S8000,
    #[strum(serialize = "11025")]
    S11025,
    #[strum(serialize = "16000")]
    S16000,
    #[strum(serialize = "22050")]
    S22050,
    #[default]
    #[strum(serialize = "44100")]
    S44100,
    #[strum(serialize = "48000")]
    S48000,
    #[strum(serialize = "88200")]
    S88200,
    #[strum(serialize = "96600")]
    S96600,
    #[strum(serialize = "176400")]
    S176400,
    #[strum(serialize = "192000")]
    S192000,
    #[strum(serialize = "352800")]
    S352800,
    #[strum(serialize = "384000")]
    S384000,
}

impl SampleRate {
    pub fn to_number(&self) -> u32 {
        self.to_string().parse().unwrap()
    }

    pub fn from_number(value: u32) -> Option<Self> {
        match value {
            8000 => Some(SampleRate::S8000),
            11025 => Some(SampleRate::S11025),
            16000 => Some(SampleRate::S16000),
            22050 => Some(SampleRate::S22050),
            44100 => Some(SampleRate::S44100),
            48000 => Some(SampleRate::S48000),
            88200 => Some(SampleRate::S88200),
            96600 => Some(SampleRate::S96600),
            176400 => Some(SampleRate::S176400),
            192000 => Some(SampleRate::S192000),
            352800 => Some(SampleRate::S352800),
            384000 => Some(SampleRate::S384000),
            _ => None,
        }
    }
}
