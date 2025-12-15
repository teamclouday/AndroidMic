#![allow(clippy::needless_range_loop)]
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use cpal::traits::StreamTrait;
use rtrb::Consumer;

use crate::{
    config::{AudioFormat, ChannelCount, Config, DenoiseKind, SampleRate},
    ui::app::{AppState, Stream},
};

mod denoise_rnnoise;
mod player;
pub mod process;
mod resampler;
mod speexdsp;

/// Audio processing parameters
#[derive(Clone, Debug)]
pub struct AudioProcessParams {
    pub target_format: AudioPacketFormat,
    pub denoise: Option<DenoiseKind>,
    pub amplify: Option<f32>,
    pub speex_noise_suppress: i32,
    pub speex_vad_enabled: bool,
    pub speex_vad_threshold: u32,
    pub speex_agc_enabled: bool,
    pub speex_agc_target: u32,
    pub speex_dereverb_enabled: bool,
    pub speex_dereverb_level: f32,
}

impl AudioProcessParams {
    pub fn new(target_format: AudioPacketFormat, config: Config) -> Self {
        Self {
            target_format,
            denoise: config.denoise.then_some(config.denoise_kind),
            amplify: config.amplify.then_some(config.amplify_value),
            speex_noise_suppress: config.speex_noise_suppress,
            speex_vad_enabled: config.speex_vad_enabled,
            speex_vad_threshold: config.speex_vad_threshold,
            speex_agc_enabled: config.speex_agc_enabled,
            speex_agc_target: config.speex_agc_target,
            speex_dereverb_enabled: config.speex_dereverb_enabled,
            speex_dereverb_level: config.speex_dereverb_level,
        }
    }

    pub fn is_speex_denoise_enabled(&self) -> bool {
        self.denoise
            .as_ref()
            .is_some_and(|k| *k == DenoiseKind::Speexdsp)
    }

    pub fn is_speex_used(&self) -> bool {
        self.is_speex_denoise_enabled()
            || self.speex_vad_enabled
            || self.speex_agc_enabled
            || self.speex_dereverb_enabled
    }
}

impl AppState {
    pub fn create_audio_stream(
        &mut self,
        consumer: Consumer<u8>,
        auto_play: bool,
    ) -> anyhow::Result<AudioPacketFormat> {
        self.audio_stream = None;

        let device = self
            .audio_device
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio device"))?;
        let config = self.config.data().clone();

        let wanted_audio_config = AudioPacketFormat {
            sample_rate: config.sample_rate,
            audio_format: config.audio_format,
            channel_count: config.channel_count,
        };

        let (stream, final_audio_config) =
            player::create_audio_stream(device, wanted_audio_config, consumer)?;

        if auto_play {
            if let Err(e) = stream.play() {
                error!("{e}");
            }
        } else if let Err(e) = stream.pause() {
            error!("{e}");
        }

        self.audio_stream = Some(Stream {
            stream,
            config: final_audio_config.clone(),
        });

        Ok(final_audio_config)
    }
}

pub trait AudioBytes {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> Vec<u8>;

    fn to_f32(&self) -> f32;

    fn from_f32(value: f32) -> Self
    where
        Self: Sized;

    fn to_f64(&self) -> f64;

    fn from_f64(value: f64) -> Self
    where
        Self: Sized;
}

impl AudioBytes for i16 {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() == 2 {
            Some(NativeEndian::read_i16(bytes))
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_i16::<NativeEndian>(*self).unwrap();
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self as f32 / i16::MAX as f32
    }

    fn from_f32(value: f32) -> Self {
        (value * i16::MAX as f32) as i16
    }

    fn to_f64(&self) -> f64 {
        *self as f64 / i16::MAX as f64
    }

    fn from_f64(value: f64) -> Self {
        (value * i16::MAX as f64) as i16
    }
}

impl AudioBytes for i32 {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() == 4 {
            Some(NativeEndian::read_i32(bytes))
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_i32::<NativeEndian>(*self).unwrap();
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self as f32 / i32::MAX as f32
    }

    fn from_f32(value: f32) -> Self {
        (value * i32::MAX as f32) as i32
    }

    fn to_f64(&self) -> f64 {
        *self as f64 / i32::MAX as f64
    }

    fn from_f64(value: f64) -> Self {
        (value * i32::MAX as f64) as i32
    }
}

impl AudioBytes for f32 {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() == 3 {
            // 3 bytes for i24 format
            let val = NativeEndian::read_i24(bytes);
            Some(val as f32 / (1 << 23) as f32)
        } else if bytes.len() == 4 {
            Some(NativeEndian::read_f32(bytes))
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_f32::<NativeEndian>(*self).unwrap();
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self
    }

    fn from_f32(value: f32) -> Self {
        value
    }

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl AudioBytes for u8 {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() == 1 {
            Some(bytes[0])
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }

    fn to_f32(&self) -> f32 {
        (*self as f32 - 128.0) / 128.0
    }

    fn from_f32(value: f32) -> Self {
        (value * 128.0 + 128.0) as u8
    }

    fn to_f64(&self) -> f64 {
        (*self as f64 - 128.0) / 128.0
    }

    fn from_f64(value: f64) -> Self {
        (value * 128.0 + 128.0) as u8
    }
}

impl AudioBytes for u32 {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if bytes.len() == 4 {
            Some(NativeEndian::read_u32(bytes))
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_u32::<NativeEndian>(*self).unwrap();
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self as f32 / u32::MAX as f32
    }

    fn from_f32(value: f32) -> Self {
        (value * u32::MAX as f32) as u32
    }

    fn to_f64(&self) -> f64 {
        *self as f64 / u32::MAX as f64
    }

    fn from_f64(value: f64) -> Self {
        (value * u32::MAX as f64) as u32
    }
}

#[derive(Debug, Clone)]
pub struct AudioPacketFormat {
    sample_rate: SampleRate,
    audio_format: AudioFormat,
    channel_count: ChannelCount,
}
