use byteorder::{ByteOrder, NativeEndian};
use rtrb::Consumer;

use crate::{
    config::{AudioFormat, ChannelCount, SampleRate},
    ui::app::AppState,
};

mod player;
pub mod resampler;

impl AppState {
    pub fn start_audio_stream(&mut self, consumer: Consumer<u8>) -> anyhow::Result<()> {
        let device = self
            .audio_device
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio device"))?;
        let config = self.config.data().clone();
        let audio_config = AudioPacketFormat {
            sample_rate: config.sample_rate,
            audio_format: config.audio_format,
            channel_count: config.channel_count,
        };

        match player::start_audio_stream(device, audio_config, consumer) {
            Ok((stream, stream_config)) => {
                self.audio_stream = Some(stream);
                self.audio_config = Some(stream_config);
            }
            Err(e) => {
                self.audio_stream = None;
                self.audio_config = None;
                return Err(e);
            }
        }

        Ok(())
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
        let mut bytes = vec![0; 2];
        NativeEndian::write_i16(&mut bytes, *self);
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self as f32 / i16::MAX as f32
    }

    fn from_f32(value: f32) -> Self {
        (value * i16::MAX as f32) as i16
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
        let mut bytes = vec![0; 4];
        NativeEndian::write_i32(&mut bytes, *self);
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self as f32 / i32::MAX as f32
    }

    fn from_f32(value: f32) -> Self {
        (value * i32::MAX as f32) as i32
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
        let mut bytes = vec![0; 4];
        NativeEndian::write_f32(&mut bytes, *self);
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self
    }

    fn from_f32(value: f32) -> Self {
        value
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
        let mut bytes = vec![0; 4];
        NativeEndian::write_u32(&mut bytes, *self);
        bytes
    }

    fn to_f32(&self) -> f32 {
        *self as f32 / u32::MAX as f32
    }

    fn from_f32(value: f32) -> Self {
        (value * u32::MAX as f32) as u32
    }
}

#[derive(Debug, Clone)]
pub struct AudioPacketFormat {
    sample_rate: SampleRate,
    audio_format: AudioFormat,
    channel_count: ChannelCount,
}
