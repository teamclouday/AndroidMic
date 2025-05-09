use std::sync::Mutex;

use anyhow::bail;
use once_cell::sync::Lazy;
use rtrb::Producer;
use rubato::Resampler;

use crate::{config::AudioFormat, streamer::AudioPacketMessage};

use super::{AudioBytes, AudioPacketFormat};

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
    // first convert audio packet to f32 vector, mono channel
    let buffer = convert_packet_to_f32_mono(&packet)?;

    // next run resampler on the buffer
    let resampled_buffer = if format.sample_rate.to_number() == packet.sample_rate {
        buffer.clone()
    } else {
        bail!("resampling not supported yet! please make sure the sample rate is the same.");
        // resample_f32_mono_stream(&buffer, packet.sample_rate, format.sample_rate.to_number())?
    };

    // finally convert to output format
    let total_bytes = resampled_buffer.len()
        * format.channel_count.to_number() as usize
        * std::mem::size_of::<F>();
    let num_bytes = std::cmp::min(producer.slots(), total_bytes);

    if num_bytes > 0 {
        match producer.write_chunk_uninit(num_bytes) {
            Ok(chunk) => {
                chunk.fill_from_iter(
                    resampled_buffer
                        .iter()
                        .take(
                            num_bytes
                                / (format.channel_count.to_number() as usize
                                    * std::mem::size_of::<F>()),
                        )
                        .flat_map(|x| {
                            std::iter::repeat(F::from_f32(*x))
                                .take(format.channel_count.to_number() as usize)
                                .flat_map(|sample| sample.to_bytes())
                        }),
                );
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

    Ok(buffer)
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
        (packet.buffer.len() / (audio_format.sample_size() * channel_count)) as usize,
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

struct ResamplerCache {
    input_rate: u32,
    output_rate: u32,
    resampler: rubato::FastFixedIn<f32>,
}

static RESAMPLER_CACHE: Lazy<Mutex<Option<ResamplerCache>>> = Lazy::new(|| Mutex::new(None));

fn resample_f32_mono_stream(
    data: &[f32],
    input_sample_rate: u32,
    output_sample_rate: u32,
) -> anyhow::Result<Vec<f32>> {
    let resample_ratio = output_sample_rate as f64 / input_sample_rate as f64;
    let mut resampler_cache = RESAMPLER_CACHE.lock().unwrap();

    if resampler_cache.is_none()
        || resampler_cache.as_ref().unwrap().input_rate != input_sample_rate
        || resampler_cache.as_ref().unwrap().output_rate != output_sample_rate
    {
        let resampler = rubato::FastFixedIn::<f32>::new(
            resample_ratio,
            1.0,
            rubato::PolynomialDegree::Cubic,
            data.len().min(512),
            1,
        )?;

        *resampler_cache = Some(ResamplerCache {
            input_rate: input_sample_rate,
            output_rate: output_sample_rate,
            resampler,
        });
    }

    let delay = resampler_cache.as_ref().unwrap().resampler.output_delay();

    let input = vec![data];
    let output = resampler_cache
        .as_mut()
        .unwrap()
        .resampler
        .process(&input, None)?
        .pop()
        .unwrap();

    Ok(output[delay..].to_vec())
}
