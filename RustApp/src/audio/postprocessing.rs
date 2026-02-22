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

#[derive(Debug, Clone, Default)]
struct AudioPostProcessor {
    channels: usize,
    sample_rate: u32,
    dry_wet_mix: f32,

    echo_delay_ms: u32,
    echo_filters: Vec<CombFilter>,

    reverb_combs: Vec<Vec<CombFilter>>,
    reverb_allpasses: Vec<Vec<AllPassFilter>>,
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
            && sample_rate == self.sample_rate
            && delay_ms == self.echo_delay_ms
        {
            for filter in &mut self.echo_filters {
                filter.decay = decay;
                filter.cutoff = cutoff;
            }

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

    pub fn configure_reverb(
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
        if channels == self.channels && sample_rate == self.sample_rate {
            for filters in &mut self.reverb_combs {
                for filter in filters {
                    filter.decay = decay;
                    filter.cutoff = cutoff;
                }
            }

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

    pub fn apply_reverb(&mut self, buffer: &mut Vec<Vec<f32>>) {
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
