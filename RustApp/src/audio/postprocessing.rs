use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct PitchShifter {
    buffer: Vec<f32>,
    write_pos: usize,
    pitch_ratio: f32,
    window_size: f32,
    phase: f32,
}

impl PitchShifter {
    fn new(sample_rate: u32, window_ms: f32, pitch_ratio: f32) -> Self {
        let window_size = (sample_rate as f32 * (window_ms / 1000.0)).max(1.0);
        let buffer_size = (window_size * 2.0).ceil() as usize;

        Self {
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            pitch_ratio,
            window_size,
            phase: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        self.buffer[self.write_pos] = input;

        // advance phase based on pitch ratio
        let phase_inc = 1.0 - self.pitch_ratio;
        self.phase += phase_inc;

        // wrap phase within window size
        if self.phase >= self.window_size {
            self.phase -= self.window_size;
        }
        if self.phase < 0.0 {
            self.phase += self.window_size;
        }

        // calculate two delay positions for crossfading
        let delay1 = self.phase;
        let mut delay2 = self.phase + (self.window_size / 2.0);
        if delay2 >= self.window_size {
            delay2 -= self.window_size;
        }

        // read delayed samples with linear interpolation
        let out1 = self.read_interpolated(delay1);
        let out2 = self.read_interpolated(delay2);

        // calculate triangle envelopes for crossfading
        let env1 = self.calculate_envelope(delay1);
        let env2 = self.calculate_envelope(delay2);

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
    fn calculate_envelope(&self, phase: f32) -> f32 {
        let half_window = self.window_size / 2.0;
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
    fn new(sample_rate: u32, center_freq: f32, q: f32) -> Self {
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
                .push(PitchShifter::new(sample_rate, window_ms, pitch_ratio));
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
                .push(BiquadFilter::new(sample_rate, center_freq, q));
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
