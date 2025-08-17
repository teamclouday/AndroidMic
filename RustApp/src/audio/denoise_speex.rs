use speexdsp::preprocess::*;

// xxx: do we really need to change the sample rate ?
// apparently, speexdsp is optimized for low sample rate (8000, 16000), according to chatgpt,
// but 16000 just doesn't work on my end
pub const DENOISE_SPEEX_SAMPLE_RATE: u32 = 48000;

pub struct DenoiseSpeexCache {
    denoisers: Vec<SpeexPreprocess>,
}

// safe because packets are processed in order, and not concurrently
unsafe impl Send for DenoiseSpeexCache {}

pub fn denoise_speex_f32_stream(
    data: &mut [Vec<i16>],
    cache: &mut Option<DenoiseSpeexCache>,
    noise_suppress: i32,
) -> anyhow::Result<()> {
    const FRAME_SIZE: usize = (DENOISE_SPEEX_SAMPLE_RATE as f32 * 0.02) as usize; // 20 ms frame

    if cache.is_none() {
        *cache = Some(DenoiseSpeexCache {
            denoisers: data
                .iter()
                .map(|_| {
                    let mut st =
                        SpeexPreprocess::new(FRAME_SIZE, DENOISE_SPEEX_SAMPLE_RATE as usize)
                            .unwrap();
                    st.set_denoise(true);
                    st.set_noise_suppress(noise_suppress);
                    st
                })
                .collect(),
        });
    }

    for (channel, st) in data
        .iter_mut()
        .zip(cache.as_mut().unwrap().denoisers.iter_mut())
    {
        for frame in channel.chunks_exact_mut(FRAME_SIZE) {
            match st.preprocess_run(frame) {
                0 => {
                    frame.fill(0);
                }
                1 => {}
                _ => panic!(),
            }
        }
    }

    Ok(())
}
