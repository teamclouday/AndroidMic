use speexdsp::preprocess::*;

pub const DENOISE_SPEEX_SAMPLE_RATE: u32 = 48000;

pub struct DenoiseSpeexCache {
    denoisers: Vec<SpeexPreprocess>,
}

// safe because ?
unsafe impl Send for DenoiseSpeexCache {}

pub fn denoise_speex_f32_stream(
    data: &mut [Vec<i16>],
    cache: &mut Option<DenoiseSpeexCache>,
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
                    st.set_noise_suppress(-30);
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
