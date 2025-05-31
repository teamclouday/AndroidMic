use anyhow::bail;
use rtrb::Producer;

use crate::{config::AudioFormat, streamer::AudioPacketMessage};

use super::{
    AudioBytes, AudioPacketFormat, denoise::denoise_f32_stream, resampler::resample_f32_stream,
};

// This function converts an audio stream from packet into producer
// apply any necessary conversions based on the audio format
// and returns mono channel f32 vector for audio wave display
pub fn convert_audio_stream(
    producer: &mut Producer<u8>,
    packet: AudioPacketMessage,
    format: &AudioPacketFormat,
) -> anyhow::Result<Vec<f32>> {
    match format.audio_format {
        AudioFormat::I16 => convert_audio_stream_internal::<i16>(producer, packet, format),
        AudioFormat::I24 => convert_audio_stream_internal::<f32>(producer, packet, format),
        AudioFormat::I32 => convert_audio_stream_internal::<i32>(producer, packet, format),
        AudioFormat::U8 => convert_audio_stream_internal::<u8>(producer, packet, format),
        AudioFormat::U32 => convert_audio_stream_internal::<u32>(producer, packet, format),
        AudioFormat::F32 => convert_audio_stream_internal::<f32>(producer, packet, format),
        _ => bail!("unsupported audio format."),
    }
    .map_err(|e| {
        warn!("failed to convert audio stream: {e}");
        e
    })
}

fn convert_audio_stream_internal<F>(
    producer: &mut Producer<u8>,
    packet: AudioPacketMessage,
    format: &AudioPacketFormat,
) -> anyhow::Result<Vec<f32>>
where
    F: cpal::SizedSample + AudioBytes + std::fmt::Debug + 'static,
{
    // first convert audio packet to f32 vector
    let buffer = convert_packet_to_f32(&packet)?;

    // prepare mono channel buffer to return
    let buffer_mono = if format.channel_count.to_number() == 1 {
        buffer[0].clone()
    } else {
        // if not mono, average the channels
        let mut mono_buffer = Vec::with_capacity(buffer[0].len());
        for i in 0..buffer[0].len() {
            let sample: f32 = buffer.iter().map(|ch| ch[i]).sum::<f32>()
                / format.channel_count.to_number() as f32;
            mono_buffer.push(sample);
        }
        mono_buffer
    };

    let enable_denoise = true;
    let denoise_sample_rate = 48000;

    // next run resampler and denoise on the buffer
    let resampled_buffer = if enable_denoise {
        let prepared_buffer = if packet.sample_rate == denoise_sample_rate {
            buffer
        } else {
            resample_f32_stream(&buffer, packet.sample_rate, denoise_sample_rate)?
        };

        // denoise the audio stream
        let denoised_buffer = denoise_f32_stream(&prepared_buffer)?;

        if format.sample_rate.to_number() == denoise_sample_rate {
            denoised_buffer
        } else {
            resample_f32_stream(
                &denoised_buffer,
                denoise_sample_rate,
                format.sample_rate.to_number(),
            )?
        }
    } else if format.sample_rate.to_number() == packet.sample_rate {
        buffer
    } else {
        resample_f32_stream(&buffer, packet.sample_rate, format.sample_rate.to_number())?
    };

    // finally convert to output format
    let num_channels = format.channel_count.to_number() as usize;
    let total_bytes: usize = resampled_buffer[0].len() * num_channels * std::mem::size_of::<F>();
    let num_bytes = std::cmp::min(producer.slots(), total_bytes);
    let num_frames = num_bytes / (num_channels * std::mem::size_of::<F>());

    if num_bytes > 0 {
        match producer.write_chunk_uninit(num_bytes) {
            Ok(chunk) => {
                let buffer_ref = &resampled_buffer;

                chunk.fill_from_iter((0..num_frames).flat_map(|frame_idx| {
                    (0..num_channels).flat_map(move |channel_idx| {
                        // compute the channel index
                        let channel = std::cmp::min(channel_idx, buffer_ref.len() - 1);
                        let sample = if frame_idx < buffer_ref[channel].len() {
                            buffer_ref[channel][frame_idx]
                        } else {
                            0.0 // fill with zero if out of bounds
                        };
                        F::from_f32(sample).to_bytes()
                    })
                }));
            }
            Err(e) => {
                warn!("dropped audio samples {e}");
            }
        };

        // warn about dropped samples
        if num_bytes < total_bytes {
            warn!("dropped {} audio bytes", total_bytes - num_bytes);
        }
    }

    Ok(buffer_mono)
}

fn convert_packet_to_f32(packet: &AudioPacketMessage) -> anyhow::Result<Vec<Vec<f32>>> {
    let audio_format = AudioFormat::from_android_format(packet.audio_format).unwrap();
    match audio_format {
        AudioFormat::U8 => convert_packet_to_f32_internal::<u8>(packet),
        AudioFormat::I16 => convert_packet_to_f32_internal::<i16>(packet),
        AudioFormat::I24 => convert_packet_to_f32_internal::<f32>(packet),
        AudioFormat::I32 => convert_packet_to_f32_internal::<i32>(packet),
        AudioFormat::F32 => convert_packet_to_f32_internal::<f32>(packet),
        _ => bail!("unsupported android audio format or sample rate."),
    }
}

fn convert_packet_to_f32_internal<F>(packet: &AudioPacketMessage) -> anyhow::Result<Vec<Vec<f32>>>
where
    F: cpal::SizedSample + AudioBytes + std::fmt::Debug + 'static,
{
    let audio_format: AudioFormat = AudioFormat::from_android_format(packet.audio_format).unwrap();
    let channel_count = packet.channel_count as usize;
    let samples_per_channel = packet.buffer.len() / (audio_format.sample_size() * channel_count);

    // Initialize a vector to hold the results for each channel
    let mut result = (0..channel_count)
        .map(|_| Vec::<f32>::with_capacity(samples_per_channel))
        .collect::<Vec<_>>();

    for buf in packet
        .buffer
        .chunks_exact(audio_format.sample_size() * channel_count)
    {
        for channel in 0..channel_count {
            let start = channel * audio_format.sample_size();
            let end = start + audio_format.sample_size();
            let sample = F::from_bytes(&buf[start..end]).unwrap().to_f32();
            result[channel].push(sample);
        }
    }

    Ok(result)
}

fn convert_packet_to_f32_mono(packet: &AudioPacketMessage) -> anyhow::Result<Vec<f32>> {
    let audio_format = AudioFormat::from_android_format(packet.audio_format).unwrap();
    match audio_format {
        AudioFormat::U8 => convert_packet_to_f32_mono_internal::<u8>(packet),
        AudioFormat::I16 => convert_packet_to_f32_mono_internal::<i16>(packet),
        AudioFormat::I24 => convert_packet_to_f32_mono_internal::<f32>(packet),
        AudioFormat::I32 => convert_packet_to_f32_mono_internal::<i32>(packet),
        AudioFormat::F32 => convert_packet_to_f32_mono_internal::<f32>(packet),
        _ => bail!("unsupported android audio format or sample rate."),
    }
}

fn convert_packet_to_f32_mono_internal<F>(packet: &AudioPacketMessage) -> anyhow::Result<Vec<f32>>
where
    F: cpal::SizedSample + AudioBytes + std::fmt::Debug + 'static,
{
    let audio_format: AudioFormat = AudioFormat::from_android_format(packet.audio_format).unwrap();
    let channel_count = packet.channel_count as usize;

    let mut result = Vec::<f32>::with_capacity(
        packet.buffer.len() / (audio_format.sample_size() * channel_count),
    );

    for buf in packet
        .buffer
        .chunks_exact(audio_format.sample_size() * channel_count)
    {
        if channel_count == 1 {
            // For mono, there's just one sample
            result.push(F::from_bytes(buf).unwrap().to_f32());
        } else {
            // For stereo, we merge the two samples into one
            let left = F::from_bytes(&buf[0..audio_format.sample_size()])
                .unwrap()
                .to_f32();
            let right = F::from_bytes(&buf[audio_format.sample_size()..])
                .unwrap()
                .to_f32();

            result.push((left + right) / 2.0); // Mix the two channels
        }
    }

    Ok(result)
}
