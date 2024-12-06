use anyhow::bail;
use byteorder::{ByteOrder, NativeEndian};
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    BuildStreamError, Device, SizedSample, StreamConfig,
};
use rtrb::{
    chunks::{ChunkError, ReadChunkIntoIter},
    Consumer,
};

use crate::{app::AppState, config::AudioFormat, map_bytes::MapBytes};

impl AppState {
    pub fn start_audio_stream(&self, consumer: Consumer<u8>) -> anyhow::Result<cpal::Stream> {
        // info!("{:?}", Endianness::native());

        let stream = self.build_audio_stream_inner::<NativeEndian>(consumer)?;

        stream.play()?;

        Ok(stream)
    }

    fn build_audio_stream_inner<E: ByteOrder>(
        &self,
        consumer: Consumer<u8>,
    ) -> anyhow::Result<cpal::Stream> {
        let Some(device) = &self.audio_device else {
            bail!("no device");
        };

        let config = self.config.data();

        let mut channel_count = config.channel_count.to_number();

        let channel_strategy = match ChannelStrategy::new(device, channel_count) {
            Some(strategy) => {
                if strategy == ChannelStrategy::MonoCloned {
                    // notify the user because it will change the printed config
                    println!("Only stereo is supported, fall back to mono cloned strategy.");
                    channel_count = 2;
                }
                strategy
            }
            None => {
                bail!("unsupported channels configuration.");
            }
        };

        let sample_rate = cpal::SampleRate(config.sample_rate.to_number());

        let stream_config = StreamConfig {
            channels: channel_count,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.audio_format {
            AudioFormat::I8 => todo!(),
            AudioFormat::I16 => build_stream::<i16, E>(
                device,
                &stream_config,
                consumer,
                channel_strategy,
                config.audio_format.sample_size(),
            ),
            AudioFormat::I24 => build_stream::<f32, E>(
                device,
                &stream_config,
                consumer,
                channel_strategy,
                config.audio_format.sample_size(),
            ),
            AudioFormat::I32 => build_stream::<i32, E>(
                device,
                &stream_config,
                consumer,
                channel_strategy,
                config.audio_format.sample_size(),
            ),
            AudioFormat::I48 => todo!(),
            AudioFormat::I64 => todo!(),
            AudioFormat::U8 => build_stream::<u8, E>(
                device,
                &stream_config,
                consumer,
                channel_strategy,
                config.audio_format.sample_size(),
            ),
            AudioFormat::U16 => todo!(),
            AudioFormat::U24 => todo!(),
            AudioFormat::U32 => todo!(),
            AudioFormat::U48 => todo!(),
            AudioFormat::U64 => todo!(),
            AudioFormat::F32 => build_stream::<f32, E>(
                device,
                &stream_config,
                consumer,
                channel_strategy,
                config.audio_format.sample_size(),
            ),
            AudioFormat::F64 => todo!(),
        }?;

        Ok(stream)
    }
}

fn build_stream<F, E>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: Consumer<u8>,
    channel_strategy: ChannelStrategy,
    sample_size: usize,
) -> Result<cpal::Stream, BuildStreamError>
where
    F: MapBytes + SizedSample + std::fmt::Debug,
    E: ByteOrder,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channel_count = config.channels as usize;
    device.build_output_stream(
        &config,
        move |data: &mut [F], _: &cpal::OutputCallbackInfo| {
            // data is the internal buf of cpal
            // we try to read the exact lenght of data from the shared buf here
            let data_size = data.len() * sample_size;
            let read_size = data_size - (data_size % sample_size); // make sure we read a multiple of sample_size
            match consumer.read_chunk(read_size) {
                Ok(chunk) => {
                    // transform the part of sharred buf into an iter
                    // only itered byte will be remove
                    let mut chunk_iter = chunk.into_iter();

                    // a frame contain sample * channel_count
                    // a sample contain a value of type Format
                    for frame in data.chunks_mut(channel_count) {
                        channel_strategy.fill_frame::<F, E>(frame, &mut chunk_iter, sample_size);
                    }
                }
                // fallback
                Err(ChunkError::TooFewSlots(available_slots)) => {
                    if available_slots == 0 {
                        return;
                    }
                    let read_size = available_slots - (available_slots % sample_size);
                    let mut chunk_iter = consumer.read_chunk(read_size).unwrap().into_iter();
                    for frame in data.chunks_mut(channel_count) {
                        channel_strategy.fill_frame::<F, E>(frame, &mut chunk_iter, sample_size);
                    }
                }
            }
        },
        err_fn,
        None, // todo: find out what this does
    )
}

#[derive(PartialEq)]
enum ChannelStrategy {
    Mono,
    Stereo,
    MonoCloned,
}

impl ChannelStrategy {
    /// in: Stereo / out: Mono -> None
    /// in: Mono / out: Stero -> MonoCloned
    /// in: Mono / out: Mono -> Mono
    /// in: Stereo / out: Stero -> Stereo
    fn new(device: &Device, channel_count: u16) -> Option<ChannelStrategy> {
        let supported_channels = device
            .supported_output_configs()
            .unwrap()
            .map(|config| config.channels());

        let mut fall_back = None;

        for supported_channel in supported_channels {
            if supported_channel == channel_count {
                match channel_count {
                    1 => return Some(ChannelStrategy::Mono),
                    2 => return Some(ChannelStrategy::Stereo),
                    _ => {}
                }
            }
            if supported_channel == 2 && channel_count == 1 {
                info!("Upmixing mono audio stream to stereo");
                fall_back = Some(ChannelStrategy::MonoCloned);
            }
        }

        fall_back
    }

    fn fill_frame<F, E>(
        &self,
        frame: &mut [F],
        chunk: &mut ReadChunkIntoIter<'_, u8>,
        sample_size: usize,
    ) where
        F: MapBytes + SizedSample + std::fmt::Debug,
        E: ByteOrder,
    {
        match self {
            ChannelStrategy::Mono => {
                if let Some(value) = F::map_bytes::<E>(chunk, sample_size) {
                    frame[0] = value;
                }
            }
            ChannelStrategy::Stereo => {
                if let Some(value) = F::map_bytes::<E>(chunk, sample_size) {
                    frame[0] = value;
                }
                if let Some(value) = F::map_bytes::<E>(chunk, sample_size) {
                    frame[1] = value;
                }
            }
            ChannelStrategy::MonoCloned => {
                if let Some(value) = F::map_bytes::<E>(chunk, sample_size) {
                    frame[0] = value;
                    frame[1] = value;
                }
            }
        }
    }
}

fn print_supported_config(device: &Device) {
    let output_configs = match device.supported_output_configs() {
        Ok(f) => f.collect(),
        Err(e) => {
            println!("Error getting supported output configs: {:?}", e);
            Vec::new()
        }
    };
    if !output_configs.is_empty() {
        println!("Supported configs:");
        for conf in output_configs.into_iter() {
            println!(
                "channel:{}, min sample rate:{}, max sample rate:{}, audio format:{}",
                conf.channels(),
                conf.min_sample_rate().0,
                conf.max_sample_rate().0,
                conf.sample_format()
            );
        }
    }
    println!();
}