use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, Default)]
struct AudioPostProcessor {
    channels: usize,
    sample_rate: u32,
    ring_buffer: Vec<Vec<f32>>,
    ring_buffer_pos: usize,
    echo_delay_ms: u32,
    echo_delay_samples: usize,
    echo_decay: f32,
    echo_cutoff: f32,
    echo_lowpass_filter: Vec<f32>,
}

impl AudioPostProcessor {
    fn configure_echo(
        &mut self,
        sample_rate: u32,
        channels: usize,
        delay_ms: u32,
        decay: f32,
        cutoff: f32,
    ) {
        if channels == self.channels
            && sample_rate == self.sample_rate
            && delay_ms == self.echo_delay_ms
        {
            return;
        }

        self.channels = channels;
        self.sample_rate = sample_rate;

        self.echo_delay_ms = delay_ms;
        self.echo_delay_samples = (sample_rate as f32 * (delay_ms as f32 / 1000.0)) as usize;
        self.echo_decay = decay;
        self.echo_cutoff = cutoff;

        self.ring_buffer = vec![vec![0.0; self.echo_delay_samples]; channels];
        self.ring_buffer_pos = 0;

        self.echo_lowpass_filter = vec![0.0; channels];
    }

    fn apply_echo(&mut self, buffer: &mut Vec<Vec<f32>>) {
        if self.echo_delay_samples == 0 || buffer.is_empty() {
            return;
        }

        let start_pos = self.ring_buffer_pos;
        let mut chunk_length = 0;

        // iterate through each channel
        for channel_idx in 0..buffer.len() {
            let channel_data = &mut buffer[channel_idx];

            let mut current_pos = start_pos;
            chunk_length = channel_data.len();

            let mut prev_filtered_state = self.echo_lowpass_filter[channel_idx];

            for sample_idx in 0..chunk_length {
                let input_sample = channel_data[sample_idx];
                let echo_sample = self.ring_buffer[channel_idx][current_pos];

                // Apply low-pass filter:
                // y_f[n] = y_f[n-1] + alpha * (x_d[n] - y_f[n-1])
                let filtered_echo =
                    prev_filtered_state + self.echo_cutoff * (echo_sample - prev_filtered_state);
                prev_filtered_state = filtered_echo;

                // Mix input with echo:
                // y[n] = x[n] + decay * y_f[n]
                let output_sample = input_sample + self.echo_decay * filtered_echo;
                channel_data[sample_idx] = output_sample;

                self.ring_buffer[channel_idx][current_pos] = output_sample;
                current_pos = (current_pos + 1) % self.echo_delay_samples;
            }

            // save final filter state for next chunk
            self.echo_lowpass_filter[channel_idx] = prev_filtered_state;
        }

        // move ring buffer position forward
        self.ring_buffer_pos = (start_pos + chunk_length) % self.echo_delay_samples;
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
) {
    let mut processor = AUDIO_POST_PROCESSOR.lock().unwrap();
    processor.configure_echo(sample_rate, buffer.len(), delay_ms, decay, cutoff);
    processor.apply_echo(buffer);
}
