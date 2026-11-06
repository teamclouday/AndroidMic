use std::time::Duration;

use anyhow::bail;
use cpal::traits::DeviceTrait;

use crate::config::{AudioFormat, ChannelCount, SampleRate};

use super::{
    AudioBytes, AudioPacketFormat,
    playout::{Playout, PlayoutReader},
};

pub fn create_audio_stream(
    device: &cpal::Device,
    config: AudioPacketFormat,
    consumer: PlayoutReader,
    target_latency: Duration,
) -> anyhow::Result<(cpal::Stream, AudioPacketFormat)> {
    let audio_format = config.audio_format.clone();
    let sample_rate = config.sample_rate.to_number();
    let mut channel_count = config.channel_count.to_number();

    // check if config is supported by device
    let supported_configs = device.supported_output_configs()?;

    let mut matched_config = None;
    for supported_config in supported_configs {
        if audio_format == supported_config.sample_format()
            && supported_config.max_sample_rate() >= sample_rate
            && supported_config.min_sample_rate() <= sample_rate
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
            matched_config = Some(supported_config);
            break;
        }
    }

    let Some(matched_config) = matched_config else {
        bail!(
            "Unsupported output audio format or sample rate. Please apply recommended format from settings page."
        );
    };

    let config = cpal::StreamConfig {
        channels: channel_count,
        sample_rate,
        buffer_size: preferred_buffer_size(&matched_config),
    };

    // create stream config
    let stream: cpal::Stream = match audio_format {
        AudioFormat::I16 => build_output_stream::<i16>(device, config, consumer, target_latency),
        AudioFormat::I24 => build_output_stream::<f32>(device, config, consumer, target_latency),
        AudioFormat::I32 => build_output_stream::<i32>(device, config, consumer, target_latency),
        AudioFormat::U8 => build_output_stream::<u8>(device, config, consumer, target_latency),
        AudioFormat::F32 => build_output_stream::<f32>(device, config, consumer, target_latency),
    }?;

    // convert stream config to AudioPacketFormat
    let config = AudioPacketFormat {
        sample_rate: SampleRate::from_number(sample_rate).unwrap(),
        audio_format,
        channel_count: ChannelCount::from_number(channel_count).unwrap(),
    };

    Ok((stream, config))
}

fn preferred_buffer_size(supported_config: &cpal::SupportedStreamConfigRange) -> cpal::BufferSize {
    const PREFERRED_FRAMES: u32 = 256;

    if let cpal::SupportedBufferSize::Range { min, max } = supported_config.buffer_size() {
        return cpal::BufferSize::Fixed(PREFERRED_FRAMES.clamp(*min, *max));
    }

    cpal::BufferSize::Default
}

fn build_output_stream<F>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    consumer: PlayoutReader,
    target_latency: Duration,
) -> anyhow::Result<cpal::Stream, cpal::Error>
where
    F: cpal::SizedSample + AudioBytes + 'static,
{
    let mut playout = Playout::new(
        consumer,
        config.channels as usize,
        config.sample_rate,
        target_latency,
    );

    device.build_output_stream(
        config,
        move |data: &mut [F], _| playout.fill(data),
        |err| error!("an error occurred on audio stream: {err}"),
        None,
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::playout::{Playout, channel, filter_delay_frames};

    #[test]
    fn output_is_silent_until_the_target_is_buffered() {
        let (mut writer, reader) = channel(4800);
        let mut playout = Playout::new(reader, 1, 48000, Duration::from_millis(10));

        for _ in 0..479 {
            writer.producer.push(0.5).unwrap();
        }
        writer.commit(479);
        let mut output = [123_i16; 64];
        playout.fill(&mut output);
        assert_eq!(output, [0; 64]);

        writer.producer.push(0.5).unwrap();
        writer.commit(1);
        // the filter delay passes first, then the signal comes through
        let mut output = [0_i16; 256];
        playout.fill(&mut output);
        assert!(
            output[..filter_delay_frames() - 8]
                .iter()
                .all(|s| s.abs() < 1000)
        );
        assert!(output[200..].iter().all(|s| (*s - 16383).abs() < 20));
    }
}
