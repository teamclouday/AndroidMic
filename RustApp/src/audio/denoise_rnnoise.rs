use nnnoiseless::DenoiseState;

use crate::audio::chunked_ring_buffer::ChunkedRingBuffer;

pub const DENOISE_RNNOISE_SAMPLE_RATE: u32 = 48000;

pub struct DenoiseCache {
    sample_buffer: Vec<ChunkedRingBuffer<f32>>,
    denoisers: Vec<Box<DenoiseState<'static>>>,
}

pub fn process_denoise_rnnoise_f32_stream(
    data: &[Vec<f32>],
    cache: &mut Option<DenoiseCache>,
) -> anyhow::Result<Vec<Vec<f32>>> {
    if match cache {
        Some(c) => data.len() != c.denoisers.len(),
        None => true,
    } {
        *cache = Some(DenoiseCache {
            sample_buffer: vec![
                ChunkedRingBuffer::new(
                    (data[0].len() / DenoiseState::FRAME_SIZE) + 1,
                    DenoiseState::FRAME_SIZE
                );
                data.len()
            ],
            denoisers: vec![DenoiseState::new(); data.len()],
        });
    }

    let cache = cache.as_mut().unwrap();
    let mut output_buffer_i16 = [0.0; DenoiseState::FRAME_SIZE];

    // Convert f32 to i16 range
    let data_i16: Vec<Vec<f32>> = data
        .iter()
        .map(|channel| channel.iter().map(|&x| x * i16::MAX as f32).collect())
        .collect();

    // Append new data into the cache
    for channel_idx in 0..data_i16.len() {
        cache.sample_buffer[channel_idx].extend(&data_i16[channel_idx]);
    }

    let mut output: Vec<Vec<f32>> =
        vec![
            Vec::with_capacity(cache.sample_buffer[0].number_of_chunk() * DenoiseState::FRAME_SIZE);
            data.len()
        ];

    while cache.sample_buffer[0].has_chunk_available() {
        for channel_idx in 0..data.len() {
            let ring_buffer = &mut cache.sample_buffer[channel_idx];
            let chunk = ring_buffer.first_chunk();

            cache.denoisers[channel_idx].process_frame(&mut output_buffer_i16, chunk);

            // Scale back to -1.0 to 1.0 range
            output[channel_idx].extend_from_slice(
                &output_buffer_i16
                    .iter()
                    .map(|&x| x / i16::MAX as f32)
                    .collect::<Vec<f32>>(),
            );
            ring_buffer.remove_first_chunk();
        }
    }

    Ok(output)
}
