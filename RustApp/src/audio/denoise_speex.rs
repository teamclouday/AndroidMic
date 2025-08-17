use speexdsp::preprocess::SpeexPreprocessConst::*;
use speexdsp::preprocess::*;

pub fn denoise_speex_f32_stream(data: &mut [Vec<i16>]) -> anyhow::Result<()> {

    let mut st = SpeexPreprocess::new(NN, 16000).unwrap();
    st.preprocess_ctl(SPEEX_PREPROCESS_SET_DENOISE, 1).unwrap();
    st.preprocess_ctl(SPEEX_PREPROCESS_SET_NOISE_SUPPRESS, -30.)
        .unwrap();

    st.preprocess_run(x)
}
