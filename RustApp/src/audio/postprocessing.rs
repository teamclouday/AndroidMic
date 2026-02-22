use std::{
    sync::{LazyLock, Mutex},
    vec,
};

#[derive(Debug, Clone, Default)]
struct SawtoothOscillator {
    phase: f32,
    phase_inc: f32,
}

impl SawtoothOscillator {
    fn new(frequency: f32, sample_rate: u32) -> Self {
        Self {
            phase: 0.0,
            phase_inc: frequency / sample_rate as f32,
        }
    }

    fn sample(&mut self) -> f32 {
        let out = self.phase * 2.0 - 1.0;
        self.phase = (self.phase + self.phase_inc) % 1.0;
        out
    }
}

#[derive(Debug, Clone, Default)]
struct EnvelopeFollower {
    attack_alpha: f32,
    release_alpha: f32,
    current_value: f32,
}

impl EnvelopeFollower {
    fn new(attack_ms: f32, release_ms: f32, sample_rate: u32) -> Self {
        let attack_alpha = (-1.0 / (attack_ms.max(0.1) * 0.001 * sample_rate as f32)).exp();
        let release_alpha = (-1.0 / (release_ms.max(0.1) * 0.001 * sample_rate as f32)).exp();
        Self {
            attack_alpha,
            release_alpha,
            current_value: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let x = input.max(0.0);
        let alpha = if x > self.current_value {
            self.attack_alpha
        } else {
            self.release_alpha
        };

        self.current_value = alpha * self.current_value + (1.0 - alpha) * x;
        self.current_value
    }
}

#[derive(Debug, Clone, Default)]
struct CombFilter {
    buffer: Vec<f32>,
    pos: usize,
    decay: f32,
    cutoff: f32,
    filter_state: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, decay: f32, cutoff: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            pos: 0,
            decay,
            cutoff,
            filter_state: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.pos];

        // Apply low-pass filter
        let filtered_echo = self.filter_state + self.cutoff * (delayed - self.filter_state);
        self.filter_state = filtered_echo;

        // Mix input with echo and write back to buffer
        let feedback = input + self.decay * filtered_echo;
        self.buffer[self.pos] = feedback;

        self.pos = (self.pos + 1) % self.buffer.len();

        delayed
    }
}

#[derive(Debug, Clone, Default)]
struct AllPassFilter {
    buffer: Vec<f32>,
    pos: usize,
    gain: f32,
}

impl AllPassFilter {
    fn new(delay_samples: usize, gain: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            pos: 0,
            gain,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.pos];

        // All-pass math
        let output = -input + delayed;
        self.buffer[self.pos] = input + (delayed * self.gain);

        self.pos = (self.pos + 1) % self.buffer.len();

        output
    }
}

#[derive(Debug, Clone, Default)]
struct PitchShifter {
    buffer: Vec<f32>,
    write_pos: usize,
    pitch_ratio: f32,
    current_window_size: f32,
    target_window_size: f32,
    phase: f32,
}

impl PitchShifter {
    fn new_fixed(sample_rate: u32, window_ms: f32, pitch_ratio: f32) -> Self {
        let window_size = (sample_rate as f32 * (window_ms / 1000.0)).max(1.0);
        let buffer_size = (window_size * 2.0).ceil() as usize;

        Self {
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            pitch_ratio,
            current_window_size: window_size,
            target_window_size: window_size,
            phase: 0.0,
        }
    }

    fn new_dynamic(sample_rate: u32, initial_window_ms: f32) -> Self {
        let max_window_ms = 100.0;
        let max_window_size = (sample_rate as f32 * (max_window_ms / 1000.0)).max(1.0);
        let buffer_size = (max_window_size * 2.0).ceil() as usize;

        let initial_window_size = (sample_rate as f32 * (initial_window_ms / 1000.0)).max(1.0);

        Self {
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            pitch_ratio: 1.0,
            current_window_size: initial_window_size,
            target_window_size: initial_window_size,
            phase: 0.0,
        }
    }

    fn set_dynamic_target(&mut self, pitch_ratio: f32, detected_hz: f32, sample_rate: u32) {
        self.pitch_ratio = pitch_ratio;

        if detected_hz > 20.0 {
            let samples_per_wave = sample_rate as f32 / detected_hz;
            let max_safe_window = (self.buffer.len() as f32 / 2.0) - 1.0;
            self.target_window_size = samples_per_wave.clamp(10.0, max_safe_window);
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        self.buffer[self.write_pos] = input;

        // smooth window size changes to avoid artifacts
        self.current_window_size += (self.target_window_size - self.current_window_size) * 0.002;

        // advance phase based on pitch ratio
        let phase_inc = 1.0 - self.pitch_ratio;
        self.phase += phase_inc;

        // wrap phase within window size
        if self.phase >= self.current_window_size {
            self.phase -= self.current_window_size;
        }
        if self.phase < 0.0 {
            self.phase += self.current_window_size;
        }

        // calculate two delay positions for crossfading
        let delay1 = self.phase;
        let mut delay2 = self.phase + (self.current_window_size / 2.0);
        if delay2 >= self.current_window_size {
            delay2 -= self.current_window_size;
        }

        // read delayed samples with linear interpolation
        let out1 = self.read_interpolated(delay1);
        let out2 = self.read_interpolated(delay2);

        // calculate triangle envelopes for crossfading
        let env1 = self.calculate_envelope(delay1, self.current_window_size);
        let env2 = self.calculate_envelope(delay2, self.current_window_size);

        // mix two delayed outputs with their envelopes
        let output = (out1 * env1) + (out2 * env2);

        // advance write position
        self.write_pos = (self.write_pos + 1) % self.buffer.len();

        output
    }

    // help function to read from buffer with linear interpolation
    fn read_interpolated(&self, delay: f32) -> f32 {
        let mut read_pos = self.write_pos as f32 - delay;
        let len_f = self.buffer.len() as f32;

        // wrap read position within buffer length
        while read_pos < 0.0 {
            read_pos += len_f;
        }
        while read_pos >= len_f {
            read_pos -= len_f;
        }

        let index1 = read_pos.floor() as usize;
        let index2 = (index1 + 1) % self.buffer.len();
        let fraction = read_pos - index1 as f32;

        let s1 = self.buffer[index1];
        let s2 = self.buffer[index2];

        // linear interpolation
        s1 + fraction * (s2 - s1)
    }

    // helper function to calculate triangle envelope for crossfading
    fn calculate_envelope(&self, phase: f32, window: f32) -> f32 {
        let half_window = window / 2.0;
        if phase < half_window {
            phase / half_window
        } else {
            1.0 - ((phase - half_window) / half_window)
        }
    }
}

#[derive(Debug, Clone, Default)]
struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    // BPF: Band-Pass Filter
    fn new_bpf(sample_rate: u32, center_freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * center_freq / (sample_rate as f32);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;

        Self {
            b0: alpha / a0,
            b1: 0.0,
            b2: -alpha / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    // LPF: Low-Pass Filter
    fn new_lpf(sample_rate: u32, cutoff_freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * cutoff_freq / (sample_rate as f32);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;

        Self {
            b0: ((1.0 - cos_w0) / 2.0) / a0,
            b1: (1.0 - cos_w0) / a0,
            b2: ((1.0 - cos_w0) / 2.0) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    // HPF: High-Pass Filter
    fn new_hpf(sample_rate: u32, cutoff_freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * cutoff_freq / (sample_rate as f32);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;

        Self {
            b0: ((1.0 + cos_w0) / 2.0) / a0,
            b1: (-(1.0 + cos_w0)) / a0,
            b2: ((1.0 + cos_w0) / 2.0) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

#[derive(Debug, Clone, Default)]
struct FlangerFilter {
    buffer: Vec<f32>,
    write_idx: usize,
    lfo_phase: f32,
    lfo_phase_inc: f32,
    min_delay_samples: f32,
    depth_samples: f32,
    feedback: f32,
}

impl FlangerFilter {
    fn new(
        sample_rate: u32,
        rate_hz: f32,
        min_delay_ms: f32,
        depth_ms: f32,
        feedback: f32,
    ) -> Self {
        let max_delay_ms = min_delay_ms + depth_ms;
        let buffer_size = ((sample_rate as f32 * max_delay_ms / 1000.0) as usize) + 10;

        Self {
            buffer: vec![0.0; buffer_size],
            write_idx: 0,
            lfo_phase: 0.0,
            lfo_phase_inc: rate_hz / sample_rate as f32,
            min_delay_samples: sample_rate as f32 * (min_delay_ms / 1000.0),
            depth_samples: sample_rate as f32 * (depth_ms / 1000.0),
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffer_len = self.buffer.len() as f32;

        // calculate current delay in samples
        let lfo_val = (self.lfo_phase * 2.0 * std::f32::consts::PI).sin();
        let lfo_mapped = lfo_val * 0.5 + 0.5;
        let current_delay = self.min_delay_samples + (self.depth_samples * lfo_mapped);

        // calculate read index with wrapping
        let mut read_idx = self.write_idx as f32 - current_delay;
        if read_idx < 0.0 {
            read_idx += buffer_len;
        }

        let idx_floor = (read_idx.floor() as usize) % self.buffer.len();
        let idx_ceil = (idx_floor + 1) % self.buffer.len();
        let fraction = read_idx.fract();

        // linear interpolation of delayed sample
        let sample_floor = self.buffer[idx_floor];
        let sample_ceil = self.buffer[idx_ceil];
        let delayed_sample = (1.0 - fraction) * sample_floor + fraction * sample_ceil;

        // write current input + feedback into buffer
        let write_val = (input + (delayed_sample * self.feedback)).clamp(-1.0, 1.0);
        self.buffer[self.write_idx] = write_val;

        // advance write index and LFO phase
        self.write_idx = (self.write_idx + 1) % self.buffer.len();
        self.lfo_phase += self.lfo_phase_inc;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        delayed_sample
    }
}

#[derive(Debug, Clone, Default)]
struct AllPassStage {
    x1: f32,
    y1: f32,
}

impl AllPassStage {
    fn process(&mut self, input: f32, a: f32) -> f32 {
        let output = a * input + self.x1 - a * self.y1;

        self.x1 = input;
        self.y1 = output;

        output
    }
}

#[derive(Debug, Clone, Default)]
struct PhaserFilter {
    stages: Vec<AllPassStage>,
    lfo_phase: f32,
    lfo_phase_inc: f32,
    f_min: f32,
    f_max: f32,
    sample_rate: f32,
    feedback: f32,
    last_output: f32,
}

impl PhaserFilter {
    fn new(
        sample_rate: u32,
        rate_hz: f32,
        f_min: f32,
        f_max: f32,
        feedback: f32,
        num_stages: usize,
    ) -> Self {
        Self {
            stages: vec![AllPassStage::default(); num_stages],
            lfo_phase: 0.0,
            lfo_phase_inc: rate_hz / sample_rate as f32,
            f_min,
            f_max,
            sample_rate: sample_rate as f32,
            feedback,
            last_output: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        // calculate LFO value for filter sweeping
        let lfo_val = (self.lfo_phase * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;

        // map LFO to current center frequency for all-pass filters
        let current_fc = self.f_min * (self.f_max / self.f_min).powf(lfo_val);

        // calculate all-pass coefficient
        let w = std::f32::consts::PI * current_fc / self.sample_rate;
        let tan_w = w.tan();
        let a = (tan_w - 1.0) / (tan_w + 1.0);

        // apply feedback from previous output
        let mut wet_signal = input + (self.last_output * self.feedback);

        // run through all-pass stages in series
        for stage in &mut self.stages {
            wet_signal = stage.process(wet_signal, a);
        }

        // store last output for feedback
        self.last_output = wet_signal.clamp(-1.0, 1.0);

        // advance LFO phase
        self.lfo_phase += self.lfo_phase_inc;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        self.last_output
    }
}

#[derive(Debug, Clone, Default)]
struct VocoderBand {
    modulator_bpf: BiquadFilter,
    carrier_bpf: BiquadFilter,
    envelope: EnvelopeFollower,
}

#[derive(Debug, Clone, Default)]
struct AudioPostProcessor {
    channels: usize,
    sample_rate: u32,
    dry_wet_mix: f32,

    echo_delay_ms: u32,
    echo_filters: Vec<CombFilter>,

    reverb_combs: Vec<Vec<CombFilter>>,
    reverb_allpasses: Vec<Vec<AllPassFilter>>,

    pitch_ratio: f32,
    pitch_shifters: Vec<PitchShifter>,

    walkie_filters: Vec<BiquadFilter>,
    walkie_drive: f32,
    walkie_center_freq: f32,
    walkie_q: f32,

    popstar_filters: Vec<PitchShifter>,
    popstar_rms_threshold: f32,
    analysis_buffer: Vec<Vec<f32>>,

    flanger_filters: Vec<FlangerFilter>,
    flanger_rate_hz: f32,
    flanger_min_delay_ms: f32,
    flanger_depth_ms: f32,

    phaser_filters: Vec<PhaserFilter>,
    phaser_rate_hz: f32,

    vocoder_bands: Vec<Vec<VocoderBand>>,
    vocoder_carriers: Vec<SawtoothOscillator>,
    vocoder_carrier_freq: f32,
    vocoder_q_factor: f32,
    vocoder_num_bands: usize,
    noise_seed: u32,
}

impl AudioPostProcessor {
    fn configure_echo(
        &mut self,
        sample_rate: u32,
        channels: usize,
        delay_ms: u32, // in milliseconds
        decay: f32,    // 0.0 to 1.0 (controls echo decay)
        cutoff: f32,   // 0.0 to 1.0 (controls high frequency loss)
        mix: f32,      // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.echo_filters.len()
            && sample_rate == self.sample_rate
            && delay_ms == self.echo_delay_ms
        {
            for filter in &mut self.echo_filters {
                filter.decay = decay;
                filter.cutoff = cutoff;
            }
            self.dry_wet_mix = mix;

            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.echo_delay_ms = delay_ms;
        let delay_samples = (sample_rate as f32 * (delay_ms as f32 / 1000.0)) as usize;
        self.echo_filters.clear();
        for _ in 0..channels {
            self.echo_filters
                .push(CombFilter::new(delay_samples, decay, cutoff));
        }
    }

    fn apply_echo(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.echo_filters.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;

        // iterate through each channel
        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let filter = &mut self.echo_filters[channel_idx];

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;
                let delayed_wet = filter.process(dry_sample);

                let output = (dry_sample * (1.0 - mix)) + (delayed_wet * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn configure_reverb(
        &mut self,
        sample_rate: u32,
        channels: usize,
        room_size: f32, // 0.0 to 1.0 (controls decay)
        damping: f32,   // 0.0 to 1.0 (controls high frequency loss)
        mix: f32,       // 0.0 to 1.0 (dry/wet blend)
    ) {
        let decay = 0.7 + (room_size * 0.25);
        let cutoff = 0.1 + ((1.0 - damping) * 0.4);

        // skip allocation if parameters haven't changed
        if channels == self.channels
            && channels == self.reverb_combs.len()
            && sample_rate == self.sample_rate
        {
            for filters in &mut self.reverb_combs {
                for filter in filters {
                    filter.decay = decay;
                    filter.cutoff = cutoff;
                }
            }
            self.dry_wet_mix = mix;

            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        // Standard Freeverb delay times (in milliseconds
        let comb_delays_ms = [35.3, 36.6, 33.8, 32.2];
        let allpass_delays_ms = [5.1, 1.7];

        self.reverb_combs.clear();
        self.reverb_allpasses.clear();

        for _ in 0..channels {
            // 4 parallel comb filters per channel
            let mut channel_combs = Vec::new();
            for &delay_ms in &comb_delays_ms {
                let delay_samples = (sample_rate as f32 * (delay_ms / 1000.0)) as usize;
                channel_combs.push(CombFilter::new(delay_samples, decay, cutoff));
            }
            self.reverb_combs.push(channel_combs);

            // 2 all-pass filters in series per channel
            let mut channel_allpasses = Vec::new();
            for &delay_ms in &allpass_delays_ms {
                let delay_samples = (sample_rate as f32 * (delay_ms / 1000.0)) as usize;
                channel_allpasses.push(AllPassFilter::new(delay_samples, 0.5));
            }
            self.reverb_allpasses.push(channel_allpasses);
        }
    }

    fn apply_reverb(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.reverb_combs.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];

            let combs = &mut self.reverb_combs[channel_idx];
            let allpasses = &mut self.reverb_allpasses[channel_idx];

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;
                let mut wet_sample = 0.0;

                // run 4 parallel comb filters and sum their output
                for comb in combs.iter_mut() {
                    wet_sample += comb.process(dry_sample);
                }

                // take average
                wet_sample *= 0.25;

                // run through all-pass filters in series
                for allpass in allpasses.iter_mut() {
                    wet_sample = allpass.process(wet_sample);
                }

                // mix dry and wet samples
                let output = (dry_sample * (1.0 - mix)) + (wet_sample * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn configure_pitch_shift(
        &mut self,
        sample_rate: u32,
        channels: usize,
        pitch_ratio: f32, // >1.0 for pitch up, <1.0 for pitch down
        mix: f32,         // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.pitch_shifters.len()
            && sample_rate == self.sample_rate
        {
            for shifter in &mut self.pitch_shifters {
                shifter.pitch_ratio = pitch_ratio;
            }
            self.dry_wet_mix = mix;
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.pitch_ratio = pitch_ratio;
        let window_ms = 40.0; // 40ms window size for pitch shifting
        self.pitch_shifters.clear();
        for _ in 0..channels {
            self.pitch_shifters
                .push(PitchShifter::new_fixed(sample_rate, window_ms, pitch_ratio));
        }
    }

    fn apply_pitch_shift(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.pitch_shifters.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let shifter = &mut self.pitch_shifters[channel_idx];

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;
                let wet_sample = shifter.process(dry_sample);

                let output = (dry_sample * (1.0 - mix)) + (wet_sample * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn configure_walkie_talkie(
        &mut self,
        sample_rate: u32,
        channels: usize,
        center_freq: f32, // Usually ~1000.0 to 1500.0 Hz
        q: f32,           // usually 1.0 to 2.0 (Quality factor for the band-pass filter)
        drive: f32,       // 1.0 to 10.0 (controls distortion amount)
        mix: f32,         // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.walkie_filters.len()
            && sample_rate == self.sample_rate
            && center_freq == self.walkie_center_freq
            && q == self.walkie_q
        {
            self.walkie_drive = drive;
            self.dry_wet_mix = mix;
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.walkie_center_freq = center_freq;
        self.walkie_q = q;
        self.walkie_drive = drive;

        self.walkie_filters.clear();
        for _ in 0..channels {
            self.walkie_filters
                .push(BiquadFilter::new_bpf(sample_rate, center_freq, q));
        }
    }

    fn apply_walkie_talkie(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.walkie_filters.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;
        let drive = self.walkie_drive;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let filter = &mut self.walkie_filters[channel_idx];

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;

                // band-pass filter
                let filtered = filter.process(dry_sample);

                // distortion using tanh waveshaper
                let distorted = (filtered * drive).tanh();

                // mix dry and wet samples
                let output = (dry_sample * (1.0 - mix)) + (distorted * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn configure_popstar(
        &mut self,
        sample_rate: u32,
        channels: usize,
        rms_threshold: f32, // 0.0 to 1.0 (minimum RMS required to attempt pitch detection)
        mix: f32,           // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.popstar_filters.len()
            && sample_rate == self.sample_rate
        {
            self.dry_wet_mix = mix;
            self.popstar_rms_threshold = rms_threshold;
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.popstar_rms_threshold = rms_threshold;
        self.popstar_filters.clear();
        self.analysis_buffer.clear();
        for _ in 0..channels {
            self.popstar_filters
                .push(PitchShifter::new_dynamic(sample_rate, 20.0));
            self.analysis_buffer
                .push(vec![0.0; (sample_rate as f32 * 0.05) as usize]); // 50ms analysis buffer
        }
    }

    fn apply_popstar(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.popstar_filters.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;
        let threshold = self.popstar_rms_threshold;
        let sample_rate = self.sample_rate;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let shifter = &mut self.popstar_filters[channel_idx];
            let history = &mut self.analysis_buffer[channel_idx];

            // update analysis buffer with current samples
            let new_len = channel_data.len();
            let history_len = history.len();
            if new_len >= history_len {
                history.copy_from_slice(&channel_data[new_len - history_len..]);
            } else {
                history.drain(0..new_len);
                history.extend_from_slice(channel_data);
            }

            // detect pitch of current buffer
            if let Some(detected_hz) = Self::detect_pitch(history, threshold, sample_rate) {
                // quantize to nearest musical note
                let target_hz = {
                    let midi_note = 69.0 + 12.0 * (detected_hz / 440.0).log2();
                    let target_note = midi_note.round();
                    440.0 * (2.0f32).powf((target_note - 69.0) / 12.0)
                };

                // calculate pitch ratio
                let pitch_ratio = target_hz / detected_hz;
                shifter.set_dynamic_target(pitch_ratio, detected_hz, sample_rate);
            } else {
                shifter.set_dynamic_target(1.0, 0.0, sample_rate);
            }

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;
                let wet_sample = shifter.process(dry_sample);

                let output = (dry_sample * (1.0 - mix)) + (wet_sample * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn detect_pitch(buffer: &[f32], threshold: f32, sample_rate: u32) -> Option<f32> {
        // check RMS to avoid pitch detection on silence
        let rms = (buffer.iter().map(|s| s * s).sum::<f32>() / buffer.len() as f32).sqrt();
        if rms < threshold {
            return None;
        }

        // define range for human voice
        let min_freq = 80.0;
        let max_freq = 1000.0;

        // convert frequency range to lag range
        let min_lag = (sample_rate as f32 / max_freq).ceil() as usize;
        let max_lag = (sample_rate as f32 / min_freq).floor() as usize;

        // ensure buffer is long enough for max lag
        if buffer.len() < max_lag {
            return None;
        }

        // auto-correlation
        let mut max_corr = 0.0;
        let mut best_lag = 0;

        for lag in min_lag..=max_lag {
            let mut corr = 0.0;

            for i in 0..(buffer.len() - lag) {
                corr += buffer[i] * buffer[i + lag];
            }

            if corr > max_corr {
                max_corr = corr;
                best_lag = lag;
            }
        }

        if best_lag > 0 {
            Some(sample_rate as f32 / best_lag as f32)
        } else {
            None
        }
    }

    fn configure_flanger(
        &mut self,
        sample_rate: u32,
        channels: usize,
        rate_hz: f32,      // 0.1 to 10.0 Hz (LFO rate)
        min_delay_ms: f32, // 0.1 to 10.0 ms (base delay time)
        depth_ms: f32,     // 0.0 to 10.0 ms (how much the delay time modulates)
        feedback: f32,     // 0.0 to 0.9 (higher values create more intense flanging)
        mix: f32,          // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.flanger_filters.len()
            && sample_rate == self.sample_rate
            && rate_hz == self.flanger_rate_hz
            && min_delay_ms == self.flanger_min_delay_ms
            && depth_ms == self.flanger_depth_ms
        {
            for flanger in &mut self.flanger_filters {
                flanger.feedback = feedback;
            }
            self.dry_wet_mix = mix;
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.flanger_rate_hz = rate_hz;
        self.flanger_min_delay_ms = min_delay_ms;
        self.flanger_depth_ms = depth_ms;

        self.flanger_filters.clear();
        for _ in 0..channels {
            self.flanger_filters.push(FlangerFilter::new(
                sample_rate,
                rate_hz,
                min_delay_ms,
                depth_ms,
                feedback,
            ));
        }
    }

    fn apply_flanger(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.flanger_filters.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let flanger = &mut self.flanger_filters[channel_idx];

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;
                let wet_sample = flanger.process(dry_sample);

                let output = (dry_sample * (1.0 - mix)) + (wet_sample * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn configure_phaser(
        &mut self,
        sample_rate: u32,
        channels: usize,
        rate_hz: f32, // 0.1 to 10.0 Hz (LFO rate)
        f_min: f32,
        f_max: f32,
        feedback: f32, // 0.0 to 0.9 (higher values create more intense phasing)
        mix: f32,      // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.phaser_filters.len()
            && sample_rate == self.sample_rate
            && rate_hz == self.phaser_rate_hz
        {
            for phaser in &mut self.phaser_filters {
                phaser.feedback = feedback;
                phaser.f_min = f_min;
                phaser.f_max = f_max;
            }
            self.dry_wet_mix = mix;
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.phaser_rate_hz = rate_hz;
        self.phaser_filters.clear();
        for _ in 0..channels {
            self.phaser_filters.push(PhaserFilter::new(
                sample_rate,
                rate_hz,
                f_min,
                f_max,
                feedback,
                6,
            ));
        }
    }

    fn apply_phaser(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.phaser_filters.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let phaser = &mut self.phaser_filters[channel_idx];

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;
                let wet_sample = phaser.process(dry_sample);

                let output = (dry_sample * (1.0 - mix)) + (wet_sample * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }

    fn configure_vocoder(
        &mut self,
        sample_rate: u32,
        channels: usize,
        num_bands: usize,  // e.g.8 to 24 bands for a clear vocoder effect
        carrier_freq: f32, // e.g. 100.0 Hz for a bassy carrier, 1000.0 Hz for a more vocal-like carrier
        q_factor: f32,     // 1.0 to 10.0 (controls bandwidth of the vocoder bands)
        mix: f32,          // 0.0 to 1.0 (dry/wet blend)
    ) {
        if channels == self.channels
            && channels == self.vocoder_bands.len()
            && channels == self.vocoder_carriers.len()
            && sample_rate == self.sample_rate
            && carrier_freq == self.vocoder_carrier_freq
            && q_factor == self.vocoder_q_factor
            && num_bands == self.vocoder_num_bands
        {
            self.dry_wet_mix = mix;
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;
        self.dry_wet_mix = mix;

        self.vocoder_carrier_freq = carrier_freq;
        self.vocoder_q_factor = q_factor;
        self.vocoder_num_bands = num_bands;

        let min_freq = 180.0f32;
        let max_freq = 6000.0f32;
        let mut center_frequencies = Vec::with_capacity(num_bands);

        if num_bands == 1 {
            center_frequencies.push((min_freq + max_freq) * 0.5);
        } else {
            for i in 0..num_bands {
                // logarithmic spacing
                let fraction = i as f32 / (num_bands as f32 - 1.0);
                let freq = min_freq * (max_freq / min_freq).powf(fraction);
                center_frequencies.push(freq);
            }
        }

        self.vocoder_bands.clear();
        self.vocoder_carriers.clear();

        for _ in 0..channels {
            self.vocoder_carriers
                .push(SawtoothOscillator::new(carrier_freq, sample_rate));

            let mut channel_bands = Vec::new();
            for &freq in &center_frequencies {
                channel_bands.push(VocoderBand {
                    modulator_bpf: BiquadFilter::new_bpf(sample_rate, freq, q_factor),
                    carrier_bpf: BiquadFilter::new_bpf(sample_rate, freq, q_factor),
                    envelope: EnvelopeFollower::new(2.0, 70.0, sample_rate),
                });
            }
            self.vocoder_bands.push(channel_bands);
        }
    }

    fn apply_vocoder(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if buffer.is_empty() || self.vocoder_bands.is_empty() {
            return;
        }

        let mix = self.dry_wet_mix;

        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];
            let carrier = &mut self.vocoder_carriers[channel_idx];
            let bands = &mut self.vocoder_bands[channel_idx];

            let band_count = bands.len() as f32;
            let split_idx = (band_count * 0.66) as usize;

            for sample in channel_data.iter_mut() {
                let dry_sample = *sample;

                // first pass: get estimate
                let mut low_env = 0.0f32;
                let mut high_env = 0.0f32;

                for (i, band) in bands.iter_mut().enumerate() {
                    let modulated = band.modulator_bpf.process(dry_sample);
                    let env = band.envelope.process(modulated.abs());

                    if i >= split_idx {
                        high_env += env;
                    } else {
                        low_env += env;
                    }
                }

                let denom = (low_env + high_env).max(1e-6);
                let unvoiced = (high_env / denom).clamp(0.0, 1.0);

                // generate white noise
                self.noise_seed = self
                    .noise_seed
                    .wrapping_mul(1664525)
                    .wrapping_add(1013904223);
                let noise = ((self.noise_seed >> 16) as f32 / 65536.0) * 2.0 - 1.0;
                let noise_mix = (0.02 + unvoiced * 0.28).clamp(0.02, 0.30);
                let carrier_sample = carrier.sample();
                let mixed_carrier = carrier_sample * (1.0 - noise_mix) + noise * noise_mix;

                // second pass: apply vocoder effect
                let mut wet_sample = 0.0;
                for band in bands.iter_mut() {
                    let modulated = band.modulator_bpf.process(dry_sample);
                    let env = band.envelope.process(modulated.abs());
                    let shaped_carrier = band.carrier_bpf.process(mixed_carrier);
                    wet_sample += shaped_carrier * env;
                }

                // normalize wet sample by number of bands and add some saturation
                wet_sample *= 1.0 / band_count.sqrt();
                wet_sample = (wet_sample * 1.35).tanh();

                // mix dry and wet samples
                let output = (dry_sample * (1.0 - mix)) + (wet_sample * mix);
                *sample = output.clamp(-1.0, 1.0);
            }
        }
    }
}

static AUDIO_POST_PROCESSOR: LazyLock<Mutex<AudioPostProcessor>> =
    LazyLock::new(|| Mutex::new(AudioPostProcessor::default()));

// apply echo effect to audio buffer in place
pub fn post_apply_echo(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    delay_ms: u32,
    decay: f32,
    cutoff: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_echo(sample_rate, buffer.len(), delay_ms, decay, cutoff, mix);
    processor.apply_echo(buffer);
}

// apply reverb effect to audio buffer in place
pub fn post_apply_reverb(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    room_size: f32,
    damping: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_reverb(sample_rate, buffer.len(), room_size, damping, mix);
    processor.apply_reverb(buffer);
}

// apply pitch shift effect to audio buffer in place
pub fn post_apply_pitch_shift(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    pitch_ratio: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_pitch_shift(sample_rate, buffer.len(), pitch_ratio, mix);
    processor.apply_pitch_shift(buffer);
}

// apply walkie-talkie effect to audio buffer in place
pub fn post_apply_walkie_talkie(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    center_freq: f32,
    q: f32,
    drive: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_walkie_talkie(sample_rate, buffer.len(), center_freq, q, drive, mix);
    processor.apply_walkie_talkie(buffer);
}

// apply popstar effect to audio buffer in place
pub fn post_apply_popstar(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    rms_threshold: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_popstar(sample_rate, buffer.len(), rms_threshold, mix);
    processor.apply_popstar(buffer);
}

// apply flanger effect to audio buffer in place
pub fn post_apply_flanger(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    rate_hz: f32,
    min_delay_ms: f32,
    depth_ms: f32,
    feedback: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_flanger(
        sample_rate,
        buffer.len(),
        rate_hz,
        min_delay_ms,
        depth_ms,
        feedback,
        mix,
    );
    processor.apply_flanger(buffer);
}

// apply phaser effect to audio buffer in place
pub fn post_apply_phaser(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    rate_hz: f32,
    f_min: f32,
    f_max: f32,
    feedback: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_phaser(
        sample_rate,
        buffer.len(),
        rate_hz,
        f_min,
        f_max,
        feedback,
        mix,
    );
    processor.apply_phaser(buffer);
}

// apply vocoder effect to audio buffer in place
pub fn post_apply_vocoder(
    buffer: &mut Vec<Vec<f32>>,
    sample_rate: u32,
    num_bands: usize,
    carrier_freq: f32,
    q_factor: f32,
    mix: f32,
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_vocoder(
        sample_rate,
        buffer.len(),
        num_bands,
        carrier_freq,
        q_factor,
        mix,
    );
    processor.apply_vocoder(buffer);
}
