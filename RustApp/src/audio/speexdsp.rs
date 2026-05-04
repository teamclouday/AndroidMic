use speexdsp::preprocess::SpeexPreprocess;

use crate::audio::{AudioProcessParams, chunked_ring_buffer::ChunkedRingBuffer};

// xxx: do we really need to change the sample rate ?
// apparently, speexdsp is optimized for low sample rate (8000, 16000), according to chatgpt,
// but 16000 just doesn't work on my end
pub const SPEEXDSP_SAMPLE_RATE: u32 = 48000;
const FRAME_SIZE: usize = (SPEEXDSP_SAMPLE_RATE as f32 * 0.02) as usize; // 20 ms frame

pub struct SpeexdspCache {
    sample_buffer: Vec<ChunkedRingBuffer<i16>>,
    denoisers: Vec<SpeexPreprocess>,
    config_denoise_enabled: bool,
    config_noise_suppress: i32,
    config_vad_enabled: bool,
    config_vad_threshold: u32,
    config_agc_enabled: bool,
    config_agc_target: u32,
    config_dereverb_enabled: bool,
    config_dereverb_level: f32,
}

// safe because packets are processed in order, and not concurrently
unsafe impl Send for SpeexdspCache {}

impl SpeexdspCache {
    fn is_config_changed(&self, config: &AudioProcessParams) -> bool {
        self.config_denoise_enabled != config.is_speex_denoise_enabled()
            || self.config_noise_suppress != config.speex_noise_suppress
            || self.config_vad_enabled != config.speex_vad_enabled
            || self.config_vad_threshold != config.speex_vad_threshold
            || self.config_agc_enabled != config.speex_agc_enabled
            || self.config_agc_target != config.speex_agc_target
            || self.config_dereverb_enabled != config.speex_dereverb_enabled
            || self.config_dereverb_level != config.speex_dereverb_level
    }
}

pub fn process_speex_f32_stream(
    data: &[Vec<f32>],
    config: &AudioProcessParams,
    cache: &mut Option<SpeexdspCache>,
) -> anyhow::Result<Vec<Vec<f32>>> {
    if match cache {
        Some(c) => {
            if data.len() != c.denoisers.len() || c.is_config_changed(config) {
                dbg!(data.len(), c.denoisers.len(), c.is_config_changed(config));
            }

            data.len() != c.denoisers.len() || c.is_config_changed(config)
        }
        None => true,
    } {
        *cache = Some(SpeexdspCache {
            sample_buffer: vec![
                ChunkedRingBuffer::new((data[0].len() / FRAME_SIZE) + 1, FRAME_SIZE);
                data.len()
            ],
            denoisers: data
                .iter()
                .map(|_| {
                    let mut st =
                        SpeexPreprocess::new(FRAME_SIZE, SPEEXDSP_SAMPLE_RATE as usize).unwrap();

                    st.set_denoise(config.is_speex_denoise_enabled());
                    st.set_noise_suppress(config.speex_noise_suppress);

                    st.set_vad(config.speex_vad_enabled);
                    st.set_prob_start(config.speex_vad_threshold.try_into().unwrap_or(i32::MAX));

                    st.set_agc(config.speex_agc_enabled);
                    st.set_agc_target(config.speex_agc_target.try_into().unwrap_or(i32::MAX));

                    st.set_dereverb(config.speex_dereverb_enabled);
                    st.set_dereverb_level(config.speex_dereverb_level);

                    st
                })
                .collect(),
            config_denoise_enabled: config.is_speex_denoise_enabled(),
            config_noise_suppress: config.speex_noise_suppress,
            config_vad_enabled: config.speex_vad_enabled,
            config_vad_threshold: config.speex_vad_threshold,
            config_agc_enabled: config.speex_agc_enabled,
            config_agc_target: config.speex_agc_target,
            config_dereverb_enabled: config.speex_dereverb_enabled,
            config_dereverb_level: config.speex_dereverb_level,
        });
    }

    let cache = cache.as_mut().unwrap();

    // Convert f32 to i16
    let data_i16: Vec<Vec<i16>> = data
        .iter()
        .map(|channel| {
            channel
                .iter()
                .map(|&x| (x * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16)
                .collect()
        })
        .collect();

    // Append new data into the cache
    for channel_idx in 0..data_i16.len() {
        cache.sample_buffer[channel_idx].extend(&data_i16[channel_idx]);
    }

    let mut output: Vec<Vec<f32>> =
        vec![Vec::with_capacity(cache.sample_buffer[0].number_of_chunk() * FRAME_SIZE); data.len()];

    while cache.sample_buffer[0].has_chunk_available() {
        for channel_idx in 0..data.len() {
            let ring_buffer = &mut cache.sample_buffer[channel_idx];

            let chunk = ring_buffer.first_chunk_mut();

            if !cache.denoisers[channel_idx].preprocess_run(chunk) {
                chunk.fill(0);
            }

            // Scale back to -1.0 to 1.0 range
            output[channel_idx].extend_from_slice(
                &chunk
                    .iter()
                    .map(|&x| x as f32 / i16::MAX as f32)
                    .collect::<Vec<f32>>(),
            );

            ring_buffer.remove_first_chunk();
        }
    }

    Ok(output)
}
