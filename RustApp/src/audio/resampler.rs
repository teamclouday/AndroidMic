use std::sync::{LazyLock, Mutex};

use rubato::Resampler;

struct ResamplerCache {
    input_rate: u32,
    output_rate: u32,
    num_channels: usize,
    sample_buffer: Vec<Vec<f32>>,
    resampler: rubato::FastFixedIn<f32>,
}

static RESAMPLER_CACHE: LazyLock<Mutex<Option<ResamplerCache>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn resample_f32_stream(
    data: &[Vec<f32>],
    input_sample_rate: u32,
    output_sample_rate: u32,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let chunk_size = 1024;
    let resample_ratio = output_sample_rate as f64 / input_sample_rate as f64;
    let mut resampler_cache = RESAMPLER_CACHE.lock().unwrap();

    if resampler_cache.is_none()
        || resampler_cache.as_ref().unwrap().input_rate != input_sample_rate
        || resampler_cache.as_ref().unwrap().output_rate != output_sample_rate
        || resampler_cache.as_ref().unwrap().num_channels != data.len()
    {
        let resampler = rubato::FastFixedIn::<f32>::new(
            resample_ratio,
            1.0,
            rubato::PolynomialDegree::Cubic,
            chunk_size,
            data.len(),
        )?;

        *resampler_cache = Some(ResamplerCache {
            input_rate: input_sample_rate,
            output_rate: output_sample_rate,
            num_channels: data.len(),
            sample_buffer: (0..data.len())
                .map(|_| Vec::with_capacity(chunk_size))
                .collect(),
            resampler,
        });
    }

    let cache = resampler_cache.as_mut().unwrap();
    let mut output: Vec<Vec<f32>> = vec![Vec::new(); data.len()];

    // Append new data into the cache
    for channel_idx in 0..data.len() {
        cache.sample_buffer[channel_idx].extend_from_slice(&data[channel_idx]);
    }

    while cache.sample_buffer[0].len() >= chunk_size {
        let chunk_output = cache.resampler.process(&cache.sample_buffer, None)?;

        // Clear the sample buffer for the next round
        for channel in &mut cache.sample_buffer {
            channel.drain(0..chunk_size);
        }

        // Append the resampled data to the output
        for (channel_idx, channel_data) in chunk_output.iter().enumerate() {
            if channel_idx < output.len() {
                output[channel_idx].extend_from_slice(channel_data);
            }
        }
    }

    // For each channel, skip the delay samples
    Ok(output)
}
