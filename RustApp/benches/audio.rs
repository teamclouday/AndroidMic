use std::io::Write;

use android_mic::{
    audio::{
        AudioPacketFormat, AudioProcessParams,
        denoise_rnnoise::{self},
        player::process_audio,
        process::{ProcessCache, convert_packet_to_f32},
        resampler::resample_f32_stream,
        speexdsp::process_speex_f32_stream,
    },
    config::{AudioEffect, AudioFormat, ChannelCount, DenoiseKind, SampleRate},
    streamer::{AudioPacketMessage, AudioStream},
};
use criterion::{Criterion, criterion_group, criterion_main};

use rtrb::RingBuffer;

fn make_random(size: usize) -> Vec<u8> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(42);

    let mut v = vec![0; size];

    rng.fill_bytes(&mut v);

    v
}

const SHARED_BUFFER_SIZE: usize = 48000 * 2;

fn bench_player(c: &mut Criterion) {
    let (mut producer, mut consumer) = RingBuffer::<u8>::new(SHARED_BUFFER_SIZE);

    // pre-fill fake audio buffer
    let mut output = vec![0i16; 480 * 2];

    let source = make_random(48000);

    c.bench_function("bench_player", |b| {
        b.iter(|| {
            producer.write(&source).unwrap();

            process_audio::<i16>(&mut output, &mut consumer, 480);
        });
    });
}

fn bench_process(c: &mut Criterion) {
    let (producer, mut consumer) = RingBuffer::<u8>::new(SHARED_BUFFER_SIZE);

    let audio_params = AudioProcessParams {
        target_format: AudioPacketFormat {
            sample_rate: SampleRate::S48000,
            audio_format: AudioFormat::I16,
            channel_count: ChannelCount::Mono,
        },
        denoise: Some(DenoiseKind::Speexdsp),
        amplify: None,
        post_effect: AudioEffect::NoEffect,
        speex_noise_suppress: -30,
        speex_vad_enabled: false,
        speex_vad_threshold: 80,
        speex_agc_enabled: false,
        speex_agc_target: 8000,
        speex_dereverb_enabled: false,
        speex_dereverb_level: 0.5,
    };

    let mut audio_stream = AudioStream::new(producer, audio_params, false);

    let mut cache = ProcessCache::new();

    c.bench_function("bench_process", |b| {
        b.iter(|| {
            let source = make_random(3840);

            let packet = AudioPacketMessage {
                buffer: source,
                sample_rate: 44100,
                channel_count: 1,
                audio_format: 2,
            };

            audio_stream
                .process_audio_packet(packet, &mut cache)
                .unwrap();

            let chunk = consumer.read_chunk(consumer.slots()).unwrap();
            chunk.commit_all();
        });
    });
}

fn bench_resampling(c: &mut Criterion) {
    let source = make_random(3840);

    let packet = AudioPacketMessage {
        buffer: source,
        sample_rate: 44100,
        channel_count: 1,
        audio_format: 2,
    };

    let buffer = convert_packet_to_f32(&packet).unwrap();

    let mut cache = None;

    c.bench_function("bench_resampling", |b| {
        b.iter(|| {
            resample_f32_stream(&buffer, 44100, 48000, &mut cache).unwrap();
        });
    });
}

fn bench_speexdsp(c: &mut Criterion) {
    let source = make_random(3840);

    let packet = AudioPacketMessage {
        buffer: source,
        sample_rate: 48000,
        channel_count: 1,
        audio_format: 2,
    };

    let buffer = convert_packet_to_f32(&packet).unwrap();

    let audio_params = AudioProcessParams {
        target_format: AudioPacketFormat {
            sample_rate: SampleRate::S48000,
            audio_format: AudioFormat::I16,
            channel_count: ChannelCount::Mono,
        },
        denoise: Some(DenoiseKind::Speexdsp),
        amplify: None,
        post_effect: AudioEffect::NoEffect,
        speex_noise_suppress: -30,
        speex_vad_enabled: false,
        speex_vad_threshold: 80,
        speex_agc_enabled: false,
        speex_agc_target: 8000,
        speex_dereverb_enabled: false,
        speex_dereverb_level: 0.5,
    };

    let mut cache = None;

    c.bench_function("bench_speexdsp", |b| {
        b.iter(|| {
            process_speex_f32_stream(&buffer, &audio_params, &mut cache).unwrap();
        });
    });
}

fn bench_rnnoise(c: &mut Criterion) {
    let source = make_random(3840);

    let packet = AudioPacketMessage {
        buffer: source,
        sample_rate: 48000,
        channel_count: 1,
        audio_format: 2,
    };

    let buffer = convert_packet_to_f32(&packet).unwrap();

    let mut cache = None;

    c.bench_function("bench_rnnoise", |b| {
        b.iter(|| {
            denoise_rnnoise::process_denoise_rnnoise_f32_stream(&buffer, &mut cache).unwrap();
        });
    });
}

#[cfg(not(target_os = "linux"))]
criterion_group!(
    benches,
    bench_player,
    bench_process,
    bench_resampling,
    bench_speexdsp,
    bench_rnnoise,
);

#[cfg(target_os = "linux")]
use pprof::criterion::{Output, PProfProfiler};

#[cfg(target_os = "linux")]
criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_player, bench_process, bench_resampling, bench_speexdsp, bench_rnnoise,
);

criterion_main!(benches);
