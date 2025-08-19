use anyhow::bail;
use cpal::traits::{DeviceTrait, StreamTrait};
use rtrb::{Consumer, chunks::ChunkError};

use crate::config::{AudioFormat, ChannelCount, SampleRate};

use super::{AudioBytes, AudioPacketFormat};

pub fn start_audio_stream(
    device: &cpal::Device,
    config: AudioPacketFormat,
    consumer: Consumer<u8>,
) -> anyhow::Result<(cpal::Stream, AudioPacketFormat)> {
    let audio_format = config.audio_format.clone();
    let sample_rate = config.sample_rate.to_number();
    let mut channel_count = config.channel_count.to_number();

    // check if config is supported by device
    let supported_configs = device.supported_output_configs()?;

    let mut supported = false;
    for supported_config in supported_configs {
        if audio_format == supported_config.sample_format()
            && supported_config.max_sample_rate().0 >= sample_rate
            && supported_config.min_sample_rate().0 <= sample_rate
        {
            // use recommended channel count
            if supported_config.channels() != channel_count {
                warn!(
                    "Using channel count {} instead of {}",
                    supported_config.channels(),
                    channel_count
                );
                channel_count = supported_config.channels();
            }
            supported = true;
            break;
        }
    }

    if !supported {
        bail!(
            "Unsupported output audio format or sample rate. Please apply recommended format from settings page."
        );
    }

    let config = cpal::StreamConfig {
        channels: channel_count,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // create stream config
    let stream: cpal::Stream = match audio_format {
        AudioFormat::I16 => build_output_stream::<i16>(device, config.clone(), consumer),
        AudioFormat::I24 => build_output_stream::<f32>(device, config.clone(), consumer),
        AudioFormat::I32 => build_output_stream::<i32>(device, config.clone(), consumer),
        AudioFormat::U8 => build_output_stream::<u8>(device, config.clone(), consumer),
        AudioFormat::F32 => build_output_stream::<f32>(device, config.clone(), consumer),
    }?;

    stream.play()?;

    // convert stream config to AudioPacketFormat
    let config = AudioPacketFormat {
        sample_rate: SampleRate::from_number(sample_rate).unwrap(),
        audio_format,
        channel_count: ChannelCount::from_number(channel_count).unwrap(),
    };

    Ok((stream, config))
}

fn build_output_stream<F>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut consumer: Consumer<u8>,
) -> anyhow::Result<cpal::Stream, cpal::BuildStreamError>
where
    F: cpal::SizedSample + AudioBytes + std::fmt::Debug + 'static,
{
    device.build_output_stream(
        &config,
        move |data: &mut [F], _| {
            // read data from the consumer
            let data_size = std::mem::size_of_val(data);
            match consumer.read_chunk(data_size) {
                Ok(chunk) => {
                    let samples = chunk
                        .into_iter()
                        .collect::<Vec<_>>()
                        .chunks_exact(std::mem::size_of::<F>())
                        .map(|chunk| F::from_bytes(chunk).unwrap())
                        .collect::<Vec<_>>();
                    let len = data.len();
                    data[..len].copy_from_slice(&samples[..len]);
                }
                Err(ChunkError::TooFewSlots(slots)) => {
                    if slots == 0 {
                        return;
                    }

                    let data_size =
                        slots - (slots % (std::mem::size_of::<F>() * config.channels as usize));
                    let chunk = consumer.read_chunk(data_size).unwrap();
                    let samples = chunk
                        .into_iter()
                        .collect::<Vec<_>>()
                        .chunks_exact(std::mem::size_of::<F>())
                        .map(|chunk| F::from_bytes(chunk).unwrap())
                        .collect::<Vec<_>>();
                    let len = samples.len().min(data.len());
                    data[..len].copy_from_slice(&samples[..len]);
                }
            }
        },
        |err| error!("an error occurred on audio stream: {err}"),
        None,
    )
}
