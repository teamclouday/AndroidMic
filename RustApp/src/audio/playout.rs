//! Fixed latency playout buffer.
//!
//! Audio arrives from the network in bursts and is captured with the phone clock, which
//! never runs at exactly the same speed as the PC sound card. A plain FIFO between the two
//! either fills up (latency grows until the buffer is full) or keeps running dry.
//!
//! This keeps the amount of buffered audio locked to a fixed target instead:
//! - playback only starts once `target` frames are buffered, and is held back a bit more if the
//!   buffer then dips below `target` (phones often deliver audio in 10 or 20 ms bursts)
//! - a slow control loop resamples by a few ppm to cancel the clock drift
//! - audio that arrives too late is replaced by silence, and skipped once it shows up,
//!   so everything after a network hiccup keeps the exact same latency as before
//!
//! The drift isn't found by watching the buffer level (it jumps with every packet and callback,
//! and a controller following it moves the latency around). Instead two long running line fits
//! give the phone sample rate (frames received over time, fitted on the network side for every
//! packet) and the sound card rate (frames played over time). Their ratio is the drift, and only
//! a weak correction is left on top to hold the latency where playback started.

use std::{
    f64::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering, fence},
    },
    time::{Duration, Instant},
};

use rtrb::{Consumer, Producer, RingBuffer};

use super::AudioBytes;

/// interpolation filter length, flat within 0.001 dB up to 20 kHz at 48 kHz
const TAPS: usize = 64;
const HALF_TAPS: usize = TAPS / 2;
const PHASES: usize = 256;
const KAISER_BETA: f64 = 9.0;
const CUTOFF: f64 = 0.94;

/// memory of the sample rate fits, in s
const TREND_TAU: f64 = 120.0;
/// how much data the fits need before we trust them, in s
const TREND_MIN_SPAN: f64 = 4.0;
/// packets this much later than the fit expects are a network hiccup, they stay out of the fit
const LATE_ARRIVAL_S: f64 = 0.05;
/// if packets keep coming late for this long, the stream itself moved: start a new fit
const REFIT_AFTER_S: f64 = 2.0;
/// packets closer than this are one burst (phones often deliver mic audio 10 or 20 ms at a time)
const BURST_GAP_S: f64 = 0.001;
/// the latency is locked once the fits are this old, before that they still move it around
const SETTLE: Duration = Duration::from_secs(60);
/// after playback starts, the lowest buffer level is watched this long to set the margin
const PROBE: Duration = Duration::from_secs(3);
/// a latency error is corrected over about this long, in s
const PHASE_CORRECTION_S: f64 = 120.0;
/// biggest clock mismatch we follow (2%)
const MAX_RATIO_OFFSET: f64 = 0.02;

/// how much audio can pile up above the target before we jump back to it
const MIN_MAX_EXCESS: Duration = Duration::from_millis(50);
/// how long we wait for late audio before giving up and starting over
const STALL_LIMIT: Duration = Duration::from_millis(500);

const LOG_INTERVAL_S: f64 = 2.0;

/// Constant latency added by the interpolation filter
pub fn filter_delay_frames() -> usize {
    HALF_TAPS - 1
}

/// Ring buffer between the network side and the player, `capacity` in samples
pub fn channel(capacity: usize) -> (PlayoutWriter, PlayoutReader) {
    let (producer, consumer) = RingBuffer::new(capacity);
    let clock = Arc::new(ArrivalClock::new());
    (
        PlayoutWriter {
            producer,
            written: 0,
            arrivals: Trend::new(TREND_TAU),
            last_packet: None,
            late_since: None,
            clock: clock.clone(),
        },
        PlayoutReader { consumer, clock },
    )
}

pub struct PlayoutWriter {
    /// interleaved samples, only whole frames
    pub producer: Producer<f32>,
    written: u64,
    /// frames received over time
    arrivals: Trend,
    /// (arrival time, frames received) of the previous packet
    last_packet: Option<(f64, f64)>,
    late_since: Option<f64>,
    clock: Arc<ArrivalClock>,
}

impl PlayoutWriter {
    /// Call after writing `frames` frames to the producer
    pub fn commit(&mut self, frames: usize) {
        self.written += frames as u64;
        let now = self.clock.now() as f64 * 1e-9;

        // only the last packet of a burst is fitted: the ones before it waited for it,
        // their arrival time says nothing about when they were captured
        if let Some((t, written)) = self.last_packet
            && now - t > BURST_GAP_S
        {
            self.fit_arrival(t, written);
        }
        self.last_packet = Some((now, self.written as f64));

        match (self.arrivals.slope(), self.arrivals.at(now)) {
            (Some(rate), Some(at_now)) => self.clock.store(Fit {
                rate,
                t: now,
                frames: at_now,
            }),
            _ => self.clock.store(Fit::NONE),
        }
    }

    fn fit_arrival(&mut self, t: f64, written: f64) {
        // how late this packet is compared to the stream so far
        let lateness = match (self.arrivals.slope(), self.arrivals.at(t)) {
            (Some(rate), Some(expected)) if rate > 0.0 => (expected - written) / rate,
            _ => 0.0,
        };

        if lateness < LATE_ARRIVAL_S {
            self.late_since = None;
            self.arrivals.add(t, written);
        } else {
            let since = *self.late_since.get_or_insert(t);
            if t - since > REFIT_AFTER_S {
                self.arrivals.reset();
                self.late_since = None;
            }
        }
    }
}

/// Line fit of the frames received over time: `frames` at time `t`, growing by `rate` per s
#[derive(Clone, Copy)]
struct Fit {
    rate: f64,
    t: f64,
    frames: f64,
}

impl Fit {
    const NONE: Fit = Fit {
        rate: f64::NAN,
        t: 0.0,
        frames: 0.0,
    };

    fn frames_at(&self, t: f64) -> Option<f64> {
        (!self.rate.is_nan()).then(|| self.frames + self.rate * (t - self.t))
    }
}

pub struct PlayoutReader {
    consumer: Consumer<f32>,
    clock: Arc<ArrivalClock>,
}

/// The current fit of the arrivals, shared by the network side and the audio callback.
/// One writer and one reader, a seqlock keeps the values consistent.
struct ArrivalClock {
    epoch: Instant,
    seq: AtomicU64,
    rate: AtomicU64,
    t: AtomicU64,
    frames: AtomicU64,
    #[cfg(test)]
    fake_now: AtomicU64,
}

impl ArrivalClock {
    fn new() -> Self {
        Self {
            epoch: Instant::now(),
            seq: AtomicU64::new(0),
            rate: AtomicU64::new(f64::NAN.to_bits()),
            t: AtomicU64::new(0),
            frames: AtomicU64::new(0),
            #[cfg(test)]
            fake_now: AtomicU64::new(u64::MAX),
        }
    }

    fn now(&self) -> u64 {
        #[cfg(test)]
        {
            let fake = self.fake_now.load(Ordering::Relaxed);
            if fake != u64::MAX {
                return fake;
            }
        }
        self.epoch.elapsed().as_nanos() as u64
    }

    fn store(&self, fit: Fit) {
        let seq = self.seq.load(Ordering::Relaxed);
        self.seq.store(seq + 1, Ordering::Relaxed);
        fence(Ordering::Release);
        self.rate.store(fit.rate.to_bits(), Ordering::Relaxed);
        self.t.store(fit.t.to_bits(), Ordering::Relaxed);
        self.frames.store(fit.frames.to_bits(), Ordering::Relaxed);
        self.seq.store(seq + 2, Ordering::Release);
    }

    fn load(&self) -> Fit {
        loop {
            let seq = self.seq.load(Ordering::Acquire);
            if seq & 1 == 0 {
                let fit = Fit {
                    rate: f64::from_bits(self.rate.load(Ordering::Relaxed)),
                    t: f64::from_bits(self.t.load(Ordering::Relaxed)),
                    frames: f64::from_bits(self.frames.load(Ordering::Relaxed)),
                };
                fence(Ordering::Acquire);
                if self.seq.load(Ordering::Relaxed) == seq {
                    return fit;
                }
            }
            std::hint::spin_loop();
        }
    }
}

pub struct Playout {
    consumer: Consumer<f32>,
    clock: Arc<ArrivalClock>,
    /// frames taken out of the ring buffer (played or skipped)
    read: u64,
    channels: usize,
    sample_rate: f64,
    /// buffered frames we aim for
    target: usize,
    max_excess: usize,
    stall_limit: usize,
    playing: bool,
    /// frames played as silence because they arrived too late, they are dropped on arrival
    debt: usize,
    /// consecutive frames that were missing
    starved: usize,
    interp: Interpolator,
    /// frames played by the sound card over time
    output: Trend,
    output_frames: u64,
    /// where the stream was when the latency got locked, see `update_ratio`
    setpoint: Option<f64>,
    /// frames played since playback started
    since_start: usize,
    settle: usize,
    probe: usize,
    /// lowest buffer level seen while probing, late audio counts as below zero
    min_level: i64,
    /// frames of silence still to play before reading again, to raise the margin
    hold: usize,
    /// last (position - setpoint), in frames
    latency_error: f64,
    /// estimated clock drift between the phone and the sound card
    drift: f64,
    ratio: f64,
    frame: Vec<f32>,
    out_frame: Vec<f32>,
    stats: Stats,
}

#[derive(Default)]
struct Stats {
    elapsed_frames: usize,
    underruns: usize,
    late_frames: usize,
    overflows: usize,
    restarts: usize,
}

impl Playout {
    pub fn new(
        reader: PlayoutReader,
        channels: usize,
        sample_rate: u32,
        target_latency: Duration,
    ) -> Self {
        let PlayoutReader { consumer, clock } = reader;
        let sr = sample_rate as f64;
        let to_frames = |d: Duration| (d.as_secs_f64() * sr).round() as usize;

        // leave room above the target, or playback could never start
        let capacity = consumer.buffer().capacity() / channels;
        let mut target = to_frames(target_latency).max(1);
        if target > capacity / 2 {
            warn!("playout: target latency is too big for the buffer, using {} frames", capacity / 2);
            target = capacity / 2;
        }

        info!(
            "playout: target latency {:.1} ms ({target} frames)",
            target_latency.as_secs_f64() * 1000.0
        );

        Self {
            consumer,
            clock,
            read: 0,
            channels,
            sample_rate: sr,
            target,
            max_excess: target.max(to_frames(MIN_MAX_EXCESS)).min(capacity - target),
            stall_limit: (2 * target).max(to_frames(STALL_LIMIT)),
            playing: false,
            debt: 0,
            starved: 0,
            interp: Interpolator::new(channels),
            output: Trend::new(TREND_TAU),
            output_frames: 0,
            setpoint: None,
            since_start: 0,
            settle: to_frames(SETTLE),
            probe: to_frames(PROBE),
            min_level: i64::MAX,
            hold: 0,
            latency_error: 0.0,
            drift: 0.0,
            ratio: 1.0,
            frame: vec![0.0; channels],
            out_frame: vec![0.0; channels],
            stats: Stats::default(),
        }
    }

    /// Fill an interleaved output buffer with the next frames
    pub fn fill<F>(&mut self, data: &mut [F])
    where
        F: AudioBytes + Copy,
    {
        let channels = self.channels;
        let frames = data.len() / channels;

        let now = self.clock.now() as f64 * 1e-9;
        self.output.add(now, self.output_frames as f64);
        self.output_frames += frames as u64;

        let mut available = self.available();

        if !self.playing {
            if available < self.target {
                data.fill(F::from_f32(0.0));
                return;
            }
            // start exactly at the target latency, the drift estimate is kept
            self.skip(available - self.target);
            available = self.target;
            self.playing = true;
            self.debt = 0;
            self.starved = 0;
            self.interp.reset();
            self.setpoint = None;
            self.since_start = 0;
            self.min_level = i64::MAX;
            self.hold = 0;
        }

        available -= self.pay_debt(available);

        if available > self.target + self.max_excess {
            // way too much audio buffered (a backlog after a stall), jump back to the target
            self.skip(available - self.target);
            self.setpoint = None;
            self.stats.overflows += 1;
        }

        if self.starved == 0 && self.debt == 0 {
            self.update_ratio(now);
        }
        let probing = self.since_start < self.probe;
        self.since_start += frames;

        for out in data.chunks_exact_mut(channels) {
            if !self.playing {
                out.fill(F::from_f32(0.0));
                continue;
            }

            self.interp.output(&mut self.out_frame);
            for (o, s) in out.iter_mut().zip(&self.out_frame) {
                *o = F::from_f32(s.clamp(-1.0, 1.0));
            }

            self.interp.frac += self.ratio;
            while self.interp.frac >= 1.0 {
                self.interp.frac -= 1.0;
                self.pull_frame();
            }
        }

        if probing && self.playing {
            // the level is lowest at the end of a callback, right before new audio comes in
            let level = self.available() as i64 - self.debt as i64;
            self.min_level = self.min_level.min(level);

            let target = self.target as i64;
            if self.since_start >= self.probe && self.min_level < target {
                debug!(
                    "playout: buffer went down to {:.1} ms, holding playback {:.1} ms",
                    self.min_level as f64 * 1000.0 / self.sample_rate,
                    (target - self.min_level) as f64 * 1000.0 / self.sample_rate
                );
                // late audio still owed is part of the level, so this covers it too
                self.hold = (target - self.min_level) as usize;
            }
        }

        self.stats.elapsed_frames += frames;
        if self.stats.elapsed_frames as f64 >= LOG_INTERVAL_S * self.sample_rate {
            self.log_stats();
        }
    }

    fn available(&self) -> usize {
        self.consumer.slots() / self.channels
    }

    fn update_ratio(&mut self, now: f64) {
        // keep the last ratio until the fits have enough data
        let fit = self.clock.load();
        let (Some(card_rate), Some(received)) = (self.output.slope(), fit.frames_at(now)) else {
            return;
        };
        let phone_rate = fit.rate;

        // frames between the (smoothed) phone capture and playback, it only changes
        // if the ratio is off, network and scheduling jitter are averaged out by the fit
        let rate = phone_rate / card_rate;
        self.drift = rate - 1.0;

        if self.since_start < self.settle {
            self.ratio = rate.clamp(1.0 - MAX_RATIO_OFFSET, 1.0 + MAX_RATIO_OFFSET);
            return;
        }

        let position = received - self.read as f64;
        let setpoint = *self.setpoint.get_or_insert(position);
        self.latency_error = position - setpoint;

        self.ratio = (rate * (1.0 + self.latency_error / (self.sample_rate * PHASE_CORRECTION_S)))
            .clamp(1.0 - MAX_RATIO_OFFSET, 1.0 + MAX_RATIO_OFFSET);
    }

    /// drop late frames that we already played as silence
    fn pay_debt(&mut self, available: usize) -> usize {
        let n = self.debt.min(available);
        if n > 0 {
            self.skip(n);
            self.debt -= n;
        }
        n
    }

    fn pull_frame(&mut self) {
        if self.hold > 0 {
            self.hold -= 1;
            self.frame.fill(0.0);
            self.interp.push(&self.frame);
            return;
        }

        if self.debt > 0 {
            let available = self.available();
            self.pay_debt(available);
        }

        match self.consumer.read_chunk(self.channels) {
            Ok(chunk) => {
                let (a, b) = chunk.as_slices();
                self.frame[..a.len()].copy_from_slice(a);
                self.frame[a.len()..].copy_from_slice(b);
                chunk.commit_all();
                self.read += 1;

                if self.starved > 0 {
                    self.starved = 0;
                }
            }
            Err(_) => {
                // the audio is late: play silence in its place and drop it when it shows up
                self.frame.fill(0.0);
                if self.starved == 0 {
                    self.stats.underruns += 1;
                }
                self.starved += 1;
                self.debt += 1;
                self.stats.late_frames += 1;

                // the stream stopped, or keeps arriving slower than we can correct
                if self.starved > self.stall_limit || self.debt > self.stall_limit {
                    // wait for the stream and buffer it again
                    self.playing = false;
                    self.debt = 0;
                    self.starved = 0;
                    self.stats.restarts += 1;
                }
            }
        }

        self.interp.push(&self.frame);
    }

    fn skip(&mut self, frames: usize) {
        if let Ok(chunk) = self.consumer.read_chunk(frames * self.channels) {
            chunk.commit_all();
            self.read += frames as u64;
        }
    }

    fn log_stats(&mut self) {
        let s = &self.stats;
        let ms = |frames: usize| frames as f64 * 1000.0 / self.sample_rate;

        if s.underruns > 0 || s.overflows > 0 || s.restarts > 0 {
            warn!(
                "playout: {} underruns ({:.1} ms late audio), {} overflows, {} restarts in the last {:.0} s. increase the target latency if this keeps happening",
                s.underruns,
                ms(s.late_frames),
                s.overflows,
                s.restarts,
                ms(s.elapsed_frames) / 1000.0,
            );
        }
        debug!(
            "playout: buffered {:.2} ms (target {:.2} ms), latency error {:.3} ms, drift {:.2} ppm, ratio {:+.2} ppm",
            ms(self.available()),
            ms(self.target),
            self.latency_error * 1000.0 / self.sample_rate,
            self.drift * 1e6,
            (self.ratio - 1.0) * 1e6
        );

        self.stats = Stats::default();
    }
}

/// Exponentially weighted least squares line fit of y over t
struct Trend {
    tau: f64,
    weight: f64,
    mean_t: f64,
    mean_y: f64,
    var_t: f64,
    cov: f64,
    first_t: f64,
    last_t: f64,
}

impl Trend {
    fn new(tau: f64) -> Self {
        Self {
            tau,
            weight: 0.0,
            mean_t: 0.0,
            mean_y: 0.0,
            var_t: 0.0,
            cov: 0.0,
            first_t: 0.0,
            last_t: 0.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new(self.tau);
    }

    fn add(&mut self, t: f64, y: f64) {
        let decay = if self.weight == 0.0 {
            self.first_t = t;
            0.0
        } else {
            (-(t - self.last_t).max(0.0) / self.tau).exp()
        };
        self.weight = self.weight * decay + 1.0;
        let k = 1.0 / self.weight;

        let dt = t - self.mean_t;
        let dy = y - self.mean_y;
        self.mean_t += k * dt;
        self.mean_y += k * dy;
        self.var_t = decay * self.var_t + dt * (t - self.mean_t);
        self.cov = decay * self.cov + dt * (y - self.mean_y);
        self.last_t = t;
    }

    fn slope(&self) -> Option<f64> {
        (self.last_t - self.first_t >= TREND_MIN_SPAN && self.var_t > 0.0)
            .then(|| self.cov / self.var_t)
    }

    fn at(&self, t: f64) -> Option<f64> {
        self.slope().map(|s| self.mean_y + s * (t - self.mean_t))
    }
}

/// Band limited fractional resampler, the ratio can change every output frame
struct Interpolator {
    channels: usize,
    /// (PHASES + 1) kernels of TAPS coefficients
    table: Vec<f32>,
    /// per channel: the last TAPS input frames, stored twice so a window is always contiguous
    hist: Vec<f32>,
    pos: usize,
    /// position of the output between hist[HALF_TAPS - 1] and hist[HALF_TAPS], in [0, 1)
    frac: f64,
}

impl Interpolator {
    fn new(channels: usize) -> Self {
        let mut table = vec![0.0; (PHASES + 1) * TAPS];

        for (phase, kernel) in table.chunks_exact_mut(TAPS).enumerate() {
            let frac = phase as f64 / PHASES as f64;
            let mut sum = 0.0;
            let mut taps = [0.0f64; TAPS];

            for (k, tap) in taps.iter_mut().enumerate() {
                let t = k as f64 - (HALF_TAPS - 1) as f64 - frac;
                let w = (1.0 - (t / HALF_TAPS as f64).powi(2)).max(0.0).sqrt();
                *tap = CUTOFF * sinc(CUTOFF * t) * bessel_i0(KAISER_BETA * w);
                sum += *tap;
            }
            // unity gain at DC
            for (k, tap) in taps.iter().enumerate() {
                kernel[k] = (tap / sum) as f32;
            }
        }

        Self {
            channels,
            table,
            hist: vec![0.0; channels * 2 * TAPS],
            pos: 0,
            frac: 0.0,
        }
    }

    fn reset(&mut self) {
        self.hist.fill(0.0);
        self.pos = 0;
        self.frac = 0.0;
    }

    fn push(&mut self, frame: &[f32]) {
        for (ch, sample) in frame.iter().enumerate() {
            let base = ch * 2 * TAPS;
            self.hist[base + self.pos] = *sample;
            self.hist[base + self.pos + TAPS] = *sample;
        }
        self.pos = (self.pos + 1) % TAPS;
    }

    fn output(&self, out: &mut [f32]) {
        let p = self.frac * PHASES as f64;
        let phase = (p as usize).min(PHASES - 1);
        let t = (p - phase as f64) as f32;

        let h0 = &self.table[phase * TAPS..(phase + 1) * TAPS];
        let h1 = &self.table[(phase + 1) * TAPS..(phase + 2) * TAPS];

        for (ch, o) in out.iter_mut().enumerate().take(self.channels) {
            let base = ch * 2 * TAPS + self.pos;
            let window = &self.hist[base..base + TAPS];

            let mut a = 0.0;
            let mut b = 0.0;
            for k in 0..TAPS {
                a += window[k] * h0[k];
                b += window[k] * h1[k];
            }
            *o = a + (b - a) * t;
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let half = x / 2.0;
    for k in 1..50 {
        term *= (half / k as f64).powi(2);
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48000;

    /// Simulate the phone capture (with its own clock and a jittery network) feeding the
    /// playout, read by a sound card with a fixed callback size. Returns the output and the
    /// input sample index of every click.
    fn simulate(
        drift_ppm: f64,
        target_ms: u64,
        packet_frames: usize,
        jitter: impl Fn(usize) -> f64,
        seconds: f64,
        click_every_s: f64,
    ) -> (Vec<f32>, Vec<f64>) {
        let (mut writer, reader) = channel(SR as usize * 2);
        let clock = writer.clock.clone();
        let mut playout = Playout::new(reader, 1, SR, Duration::from_millis(target_ms));

        let input_rate = SR as f64 * (1.0 + drift_ppm * 1e-6);
        let total_in = (seconds * input_rate) as usize;
        let click_every = click_every_s * input_rate;

        // capture side: the input sample stream, with a click every `click_every_s`
        let mut input = vec![0.0f32; total_in];
        let mut clicks = Vec::new();
        let mut next = click_every;
        while (next as usize) < total_in {
            input[next as usize] = 0.8;
            clicks.push(next.floor());
            next += click_every;
        }

        // packet n is captured at (n + 1) * packet_frames / input_rate and arrives later
        let packets: Vec<(f64, usize)> = (0..total_in / packet_frames)
            .map(|n| {
                let captured = ((n + 1) * packet_frames) as f64 / input_rate;
                (captured + 0.002 + jitter(n), n)
            })
            .collect();
        let mut arrival = packets.clone();
        // tcp: packets are delivered in order, a late one holds the following ones back
        for i in 1..arrival.len() {
            if arrival[i].0 < arrival[i - 1].0 {
                arrival[i].0 = arrival[i - 1].0;
            }
        }

        let callback = 256;
        let total_out = (seconds * SR as f64) as usize;
        let mut output = Vec::with_capacity(total_out);
        let mut buf = vec![0.0f32; callback];
        let mut next_packet = 0;

        while output.len() + callback <= total_out {
            let now = output.len() as f64 / SR as f64;
            while next_packet < arrival.len() && arrival[next_packet].0 <= now {
                let (at, n) = arrival[next_packet];
                for s in &input[n * packet_frames..(n + 1) * packet_frames] {
                    let _ = writer.producer.push(*s);
                }
                clock.fake_now.store((at * 1e9) as u64, Ordering::Relaxed);
                writer.commit(packet_frames);
                next_packet += 1;
            }
            clock.fake_now.store((now * 1e9) as u64, Ordering::Relaxed);
            playout.fill(&mut buf);
            output.extend_from_slice(&buf);
        }

        (output, clicks)
    }

    /// latency (output time - capture time, in ms) of every click found in the output
    fn click_latencies(output: &[f32], clicks: &[f64], drift_ppm: f64) -> Vec<f64> {
        let input_rate = SR as f64 * (1.0 + drift_ppm * 1e-6);
        let mut latencies = Vec::new();
        let mut i = 0;
        let mut click = 0;

        while i < output.len() && click < clicks.len() {
            if output[i].abs() > 0.3 {
                // refine the peak position with a parabola over the 3 biggest samples
                let peak = (i..(i + TAPS).min(output.len()))
                    .max_by(|a, b| output[*a].abs().total_cmp(&output[*b].abs()))
                    .unwrap();
                let (y0, y1, y2) = (output[peak - 1], output[peak], output[peak + 1]);
                let offset = 0.5 * (y0 - y2) / (y0 - 2.0 * y1 + y2);
                let out_time = (peak as f64 + offset as f64) / SR as f64;

                // match the click that was captured right before
                while click + 1 < clicks.len() && clicks[click + 1] / input_rate < out_time {
                    click += 1;
                }
                latencies.push((out_time - clicks[click] / input_rate) * 1000.0);
                click += 1;
                i = peak + TAPS;
            } else {
                i += 1;
            }
        }
        latencies
    }

    /// biggest change of latency between two consecutive clicks, i.e. how much the time
    /// between two clicks at the output differs from the time between them at the mic
    fn max_interval_error(latencies: &[f64]) -> f64 {
        latencies
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f64::max)
    }

    fn spread(v: &[f64]) -> f64 {
        let min = v.iter().cloned().fold(f64::MAX, f64::min);
        let max = v.iter().cloned().fold(f64::MIN, f64::max);
        max - min
    }

    #[test]
    fn latency_stays_constant_with_drift_and_jitter() {
        for drift in [-200.0, -30.0, 0.0, 50.0, 300.0] {
            // pseudo random jitter up to 15 ms, with a 30 ms spike every second
            let jitter = |n: usize| {
                let r = ((n as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 40) as f64
                    / (1u64 << 24) as f64;
                r * 0.015 + if n % 200 == 0 { 0.03 } else { 0.0 }
            };
            let (output, clicks) = simulate(drift, 40, 240, jitter, 120.0, 0.20453);
            let latencies = click_latencies(&output, &clicks, drift);

            // let the drift loop settle, the simulated phone stops sending at the very end
            let settled = &latencies[latencies.len() / 2..latencies.len() - 25];
            assert!(settled.len() > 200, "found {} clicks", settled.len());
            let s = spread(settled);
            let interval = max_interval_error(settled);
            println!(
                "drift {drift} ppm: latency {:.3} ms, spread {:.4} ms, worst interval error {:.4} ms",
                settled[0], s, interval
            );
            assert!(s < 0.2, "latency moved by {s} ms with {drift} ppm drift");
            // clicks 204.53 ms apart at the mic stay 204.53 ms apart, within a sample
            assert!(interval < 0.02, "interval off by {interval} ms with {drift} ppm drift");
        }
    }

    #[test]
    fn late_audio_does_not_shift_the_timeline() {
        // a 150 ms network stall at 40 s, the delayed packets then arrive all at once
        let stall = 40.0 * SR as f64 / 240.0;
        let stall = stall as usize..stall as usize + 30;
        let jitter = |n: usize| {
            if stall.contains(&n) {
                0.15 - (n - stall.start) as f64 * 0.005
            } else {
                0.0
            }
        };
        let (output, clicks) = simulate(0.0, 30, 240, jitter, 70.0, 0.1);
        let latencies = click_latencies(&output, &clicks, 0.0);

        // the clicks inside the stall are lost, all the others are played on time
        assert!(latencies.len() + 4 >= clicks.len(), "{} {}", latencies.len(), clicks.len());

        let before = &latencies[250..390];
        let after = &latencies[latencies.len() - 250..latencies.len() - 50];
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        println!(
            "before stall {:.3} ms (spread {:.4}), after {:.3} ms (spread {:.4})",
            mean(before),
            spread(before),
            mean(after),
            spread(after)
        );
        assert!(spread(before) < 0.2);
        assert!(spread(after) < 0.2);
        assert!((mean(before) - mean(after)).abs() < 0.1);
    }

    #[test]
    fn trend_finds_the_rate() {
        let mut trend = Trend::new(TREND_TAU);
        // 48 kHz + 37 ppm, 5 ms packets with up to 3 ms of one sided jitter
        let rate = 48000.0 * (1.0 + 37e-6);
        for n in 0..12000u64 {
            let t = 1000.0 + n as f64 * 0.005;
            let jitter = ((n.wrapping_mul(2654435761) >> 8) % 3000) as f64 * 1e-6;
            trend.add(t + jitter, t * rate);
        }
        let slope = trend.slope().unwrap();
        let ppm = (slope / 48000.0 - 1.0) * 1e6;
        println!("estimated {ppm:.3} ppm");
        assert!((ppm - 37.0).abs() < 0.5, "{ppm}");
    }

    #[test]
    fn interpolator_is_transparent() {
        let mut interp = Interpolator::new(1);
        let mut out = [0.0f32];
        let freq = 15000.0 / SR as f64;
        let mut err = 0.0f64;
        let mut sig = 0.0f64;
        let ratio = 1.0 + 123e-6;
        let mut read = 0usize;
        let mut written = 0usize;

        for _ in 0..48000 {
            interp.output(&mut out);
            // output position in input frames, accounting for the filter delay
            let t = read as f64 - TAPS as f64 + HALF_TAPS as f64 - 1.0 + interp.frac;
            if written > 1000 {
                let expected = (2.0 * PI * freq * t).sin();
                err += (out[0] as f64 - expected).powi(2);
                sig += expected.powi(2);
            }
            written += 1;
            interp.frac += ratio;
            while interp.frac >= 1.0 {
                interp.frac -= 1.0;
                interp.push(&[(2.0 * PI * freq * read as f64).sin() as f32]);
                read += 1;
            }
        }
        let snr = 10.0 * (sig / err).log10();
        println!("snr at 15 kHz: {snr:.1} dB");
        assert!(snr > 80.0, "snr {snr}");
    }
}
