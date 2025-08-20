use std::sync::{LazyLock, Mutex};

use speexdsp::preprocess::{SpeexPreprocess, SpeexPreprocessConst};

use crate::audio::AudioProcessParams;

// xxx: do we really need to change the sample rate ?
// apparently, speexdsp is optimized for low sample rate (8000, 16000), according to chatgpt,
// but 16000 just doesn't work on my end
pub const SPEEXDSP_SAMPLE_RATE: u32 = 48000;
const FRAME_SIZE: usize = (SPEEXDSP_SAMPLE_RATE as f32 * 0.02) as usize; // 20 ms frame

struct SpeexdspCache {
    sample_buffer: Vec<Vec<i16>>,
    denoisers: Vec<SpeexPreprocess>,
    config_noise_suppress: i32,
    config_vad_enabled: bool,
    config_vad_threshold: u32,
    config_agc_enabled: bool,
    config_agc_target: u32,
    config_dereverb_enabled: bool,
    config_dereverb_level: f32,
}

impl SpeexdspCache {
    fn is_config_changed(&self, config: &AudioProcessParams) -> bool {
        self.config_noise_suppress != config.speex_noise_suppress
            || self.config_vad_enabled != config.speex_vad_enabled
            || self.config_vad_threshold != config.speex_vad_threshold
            || self.config_agc_enabled != config.speex_agc_enabled
            || self.config_agc_target != config.speex_agc_target
            || self.config_dereverb_enabled != config.speex_dereverb_enabled
            || self.config_dereverb_level != config.speex_dereverb_level
    }
}

// safe because packets are processed in order, and not concurrently
unsafe impl Send for SpeexdspCache {}

// safe because packets are processed in order, and not concurrently
static DENOISE_CACHE: LazyLock<Mutex<Option<SpeexdspCache>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn process_speex_f32_stream(
    data: &[Vec<f32>],
    config: &AudioProcessParams,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let mut denoise_cache = DENOISE_CACHE.lock().unwrap();

    if denoise_cache.is_none()
        || data.len() != denoise_cache.as_ref().unwrap().denoisers.len()
        || denoise_cache.as_ref().unwrap().is_config_changed(config)
    {
        *denoise_cache = Some(SpeexdspCache {
            sample_buffer: vec![Vec::with_capacity(FRAME_SIZE); data.len()],
            denoisers: data
                .iter()
                .map(|_| {
                    let mut st =
                        SpeexPreprocess::new(FRAME_SIZE, SPEEXDSP_SAMPLE_RATE as usize).unwrap();

                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_DENOISE,
                        if config.is_speex_denoise_enabled() {
                            1
                        } else {
                            0
                        },
                    )
                    .unwrap();

                    st.set_noise_suppress(config.speex_noise_suppress);

                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_VAD,
                        if config.speex_vad_enabled { 1 } else { 0 },
                    )
                    .unwrap();
                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_PROB_START,
                        config.speex_vad_threshold,
                    )
                    .unwrap();
                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_AGC,
                        if config.speex_agc_enabled { 1 } else { 0 },
                    )
                    .unwrap();
                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_AGC_TARGET,
                        config.speex_agc_target,
                    )
                    .unwrap();
                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_DEREVERB,
                        if config.speex_dereverb_enabled { 1 } else { 0 },
                    )
                    .unwrap();
                    st.preprocess_ctl(
                        SpeexPreprocessConst::SPEEX_PREPROCESS_SET_DEREVERB_LEVEL,
                        config.speex_dereverb_level,
                    )
                    .unwrap();
                    st
                })
                .collect(),
            config_noise_suppress: config.speex_noise_suppress,
            config_vad_enabled: config.speex_vad_enabled,
            config_vad_threshold: config.speex_vad_threshold,
            config_agc_enabled: config.speex_agc_enabled,
            config_agc_target: config.speex_agc_target,
            config_dereverb_enabled: config.speex_dereverb_enabled,
            config_dereverb_level: config.speex_dereverb_level,
        });
    }

    let cache = denoise_cache.as_mut().unwrap();
    let mut output: Vec<Vec<f32>> = vec![Vec::new(); data.len()];

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
        cache.sample_buffer[channel_idx].extend_from_slice(&data_i16[channel_idx]);
    }

    while cache.sample_buffer[0].len() >= FRAME_SIZE {
        for channel_idx in 0..data.len() {
            match cache.denoisers[channel_idx]
                .preprocess_run(&mut cache.sample_buffer[channel_idx][0..FRAME_SIZE])
            {
                0 => {
                    cache.sample_buffer[channel_idx][0..FRAME_SIZE].fill(0);
                }
                1 => {}
                _ => panic!(),
            }

            // Scale back to -1.0 to 1.0 range
            output[channel_idx].extend_from_slice(
                &cache.sample_buffer[channel_idx][0..FRAME_SIZE]
                    .iter()
                    .map(|&x| x as f32 / i16::MAX as f32)
                    .collect::<Vec<f32>>(),
            );
        }

        // Clear the sample buffer for the next round
        for channel in &mut cache.sample_buffer {
            channel.drain(0..FRAME_SIZE);
        }
    }

    Ok(output)
}
