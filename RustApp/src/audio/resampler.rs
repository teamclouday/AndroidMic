use rubato::{Indexing, Resampler};

pub struct ResamplerCache {
    input_rate: usize,
    output_rate: usize,
    nb_channels: usize,
    unprocessed_buffer: Vec<Vec<f32>>,
    resampler: rubato::Fft<f32>,
    result: Vec<Vec<f32>>,
}

const CHUNK_SIZE: usize = 1024;

pub fn resample_f32_stream_owned(
    data: &[Vec<f32>],
    input_sample_rate: usize,
    output_sample_rate: usize,
    cache: &mut Option<ResamplerCache>,
) -> anyhow::Result<Vec<Vec<f32>>> {
    resample_f32_stream(data, input_sample_rate, output_sample_rate, cache)?;

    let cache = cache.as_mut().unwrap();

    Ok(std::mem::replace(
        &mut cache.result,
        vec![Vec::new(); cache.nb_channels],
    ))
}

pub fn resample_f32_stream<'a>(
    data: &[Vec<f32>],
    input_sample_rate: usize,
    output_sample_rate: usize,
    cache: &'a mut Option<ResamplerCache>,
) -> anyhow::Result<&'a mut [Vec<f32>]> {
    let input_len = data[0].len();
    let nb_channel = data.len();

    if match cache {
        Some(c) => {
            c.input_rate != input_sample_rate
                || c.output_rate != output_sample_rate
                || c.nb_channels != data.len()
        }
        None => true,
    } {
        let resampler = rubato::Fft::new(
            input_sample_rate,
            output_sample_rate,
            CHUNK_SIZE,
            1,
            nb_channel,
            rubato::FixedSync::Both,
        )?;

        *cache = Some(ResamplerCache {
            input_rate: input_sample_rate,
            output_rate: output_sample_rate,
            nb_channels: nb_channel,
            unprocessed_buffer: vec![Vec::with_capacity(resampler.input_frames_max()); nb_channel],
            resampler,
            result: vec![Vec::new(); nb_channel],
        });
    };

    let cache = cache.as_mut().unwrap();
    let ResamplerCache {
        resampler,
        unprocessed_buffer,
        ..
    } = cache;

    let needed = resampler.input_frames_next();

    let output_len = {
        let max_output_frames = cache.resampler.output_frames_max();
        let estimated_chunks = (input_len + unprocessed_buffer[0].len()) / needed + 2;
        max_output_frames * estimated_chunks
    };

    for v in &mut cache.result {
        if v.capacity() < output_len {
            v.reserve_exact(output_len - v.len());
        }
        // safety: we only write in this function and we set the real len at the end
        unsafe {
            v.set_len(output_len);
        }
    }

    let mut buffer_out = rubato::audioadapter_buffers::direct::SequentialSliceOfVecs::new_mut(
        &mut cache.result,
        nb_channel,
        output_len,
    )
    .unwrap();

    let mut input_offset = 0;
    let mut output_offset = 0;

    if !unprocessed_buffer[0].is_empty() {
        let missing = needed - unprocessed_buffer[0].len();
        if input_len >= missing {
            for (data, unprocessed_buffer) in data.iter().zip(unprocessed_buffer.iter_mut()) {
                unprocessed_buffer.extend_from_slice(&data[0..missing]);
            }

            let buffer_in_leftover =
                rubato::audioadapter_buffers::direct::SequentialSliceOfVecs::new(
                    unprocessed_buffer,
                    nb_channel,
                    needed,
                )
                .unwrap();

            let (_, frames) = cache
                .resampler
                .process_into_buffer(&buffer_in_leftover, &mut buffer_out, None)
                .unwrap();

            output_offset = frames;

            for channel in unprocessed_buffer.iter_mut() {
                channel.clear();
            }
            input_offset = missing;
        }
    };

    // workarround for the lifetime issue
    // because in process_into_buffer, buffer_out and buffer_in must have the same lifetime.
    let mut buffer_out = rubato::audioadapter_buffers::direct::SequentialSliceOfVecs::new_mut(
        &mut cache.result,
        nb_channel,
        output_len,
    )
    .unwrap();

    let buffer_in = rubato::audioadapter_buffers::direct::SequentialSliceOfVecs::new(
        data, nb_channel, input_len,
    )
    .unwrap();

    while (input_len - input_offset) >= needed {
        let indexing = Indexing {
            input_offset,
            output_offset,
            partial_len: None,
            active_channels_mask: None,
        };

        let (_, frames) = cache
            .resampler
            .process_into_buffer(&buffer_in, &mut buffer_out, Some(&indexing))
            .unwrap();

        input_offset += needed;
        output_offset += frames;
    }

    for (chunk, unprocessed_buffer) in data.iter().zip(unprocessed_buffer.iter_mut()) {
        unprocessed_buffer.clear();
        unprocessed_buffer.extend_from_slice(&chunk[input_offset..]);
    }

    for channel in &mut cache.result {
        channel.truncate(output_offset);
    }

    Ok(&mut cache.result)
}
