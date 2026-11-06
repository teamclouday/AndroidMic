use std::borrow::Cow;

use crate::{
    audio::{
        denoise_rnnoise::{DENOISE_RNNOISE_SAMPLE_RATE, DenoiseCache},
        postprocessing::{
            post_apply_echo, post_apply_flanger, post_apply_phaser, post_apply_pitch_shift,
            post_apply_popstar, post_apply_reverb, post_apply_robot, post_apply_walkie_talkie,
        },
        resampler::{ResamplerCache, resample_f32_stream_owned},
        speexdsp::{SPEEXDSP_SAMPLE_RATE, SpeexdspCache, process_speex_f32_stream},
    },
    config::{AudioEffect, AudioFormat, DenoiseKind},
    streamer::{AudioPacketMessage, AudioStream},
};

use super::{
    AudioBytes, denoise_rnnoise::process_denoise_rnnoise_f32_stream, resampler::resample_f32_stream,
};

#[derive(Default)]
pub struct ProcessCache {
    resample_rnnoise_cache: Option<ResamplerCache>,
    resample_speexdsp_cache: Option<ResamplerCache>,
    resample_to_target: Option<ResamplerCache>,
    speexdsp: Option<SpeexdspCache>,
    denoise: Option<DenoiseCache>,
}

impl ProcessCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        self.resample_rnnoise_cache = None;
        self.resample_speexdsp_cache = None;
        self.resample_to_target = None;
        self.speexdsp = None;
        self.denoise = None;
    }
}

impl AudioStream {
    /// This function converts an audio stream from packet into producer
    /// apply any necessary conversions based on the audio format
    /// and returns mono channel f32 vector for audio wave display
    pub fn process_audio_packet(
        &mut self,
        packet: AudioPacketMessage,
        cache: &mut ProcessCache,
    ) -> anyhow::Result<Option<Vec<f32>>> {
        self.process_audio_packet_internal(packet, cache)
            .map_err(|e| {
                warn!("failed to convert audio stream: {e}");
                e
            })
    }

    /// Fill the gap left by lost packets with silence, so the audio after it is
    /// still played on time. `frames` is counted at the phone sample rate.
    pub fn push_silence(&mut self, frames: usize, sample_rate: u32) {
        let target_rate = self.audio_params.target_format.sample_rate.to_number();
        let num_channels = self.audio_params.target_format.channel_count.to_number() as usize;
        let frames = (frames as u64 * target_rate as u64 / sample_rate.max(1) as u64) as usize;
        let frames = frames.min(self.buff.producer.slots() / num_channels);

        if let Ok(chunk) = self.buff.producer.write_chunk_uninit(frames * num_channels) {
            chunk.fill_from_iter(std::iter::repeat(0.0));
            self.buff.commit(frames);
        }
    }

    fn process_audio_packet_internal(
        &mut self,
        packet: AudioPacketMessage,
        cache: &mut ProcessCache,
    ) -> anyhow::Result<Option<Vec<f32>>> {
        let config = &self.audio_params;
        let mut current_sample_rate = packet.sample_rate;

        // first convert audio packet to f32 vector
        let mut buffer = convert_packet_to_f32(&packet)?;

        // next run resampler and denoise on the buffer
        if let Some(denoise) = &config.denoise {
            match denoise {
                DenoiseKind::Rnnoise => {
                    let prepared_buffer: Cow<'_, [Vec<f32>]> =
                        if current_sample_rate == DENOISE_RNNOISE_SAMPLE_RATE {
                            Cow::Borrowed(&buffer)
                        } else {
                            let tmp = resample_f32_stream(
                                &buffer,
                                current_sample_rate as usize,
                                DENOISE_RNNOISE_SAMPLE_RATE as usize,
                                &mut cache.resample_rnnoise_cache,
                            )?;
                            current_sample_rate = DENOISE_RNNOISE_SAMPLE_RATE;
                            Cow::Borrowed(tmp)
                        };

                    // denoise the audio stream
                    buffer =
                        process_denoise_rnnoise_f32_stream(&prepared_buffer, &mut cache.denoise)?;
                }
                DenoiseKind::Speexdsp => {}
            }
        }

        if config.is_speex_used() {
            let prepared_buffer: Cow<'_, [Vec<f32>]> =
                if current_sample_rate == SPEEXDSP_SAMPLE_RATE {
                    Cow::Borrowed(&buffer)
                } else {
                    let tmp = resample_f32_stream(
                        &buffer,
                        current_sample_rate as usize,
                        SPEEXDSP_SAMPLE_RATE as usize,
                        &mut cache.resample_speexdsp_cache,
                    )?;
                    current_sample_rate = SPEEXDSP_SAMPLE_RATE;
                    Cow::Borrowed(tmp)
                };

            buffer = process_speex_f32_stream(&prepared_buffer, config, &mut cache.speexdsp)?;
        }

        buffer = if config.target_format.sample_rate.to_number() == current_sample_rate {
            buffer
        } else {
            resample_f32_stream_owned(
                &buffer,
                current_sample_rate as usize,
                config.target_format.sample_rate.to_number() as usize,
                &mut cache.resample_to_target,
            )?
        };

        // inject post effect if needed
        // NOTE: one day I might add UI for users to customize these parameters, but for now just hardcode the presets
        match &config.post_effect {
            AudioEffect::NoEffect => {}
            AudioEffect::Echo => {
                post_apply_echo(&mut buffer, current_sample_rate, 300, 0.5, 0.3, 0.25);
            }
            AudioEffect::ReverbIntimate => {
                post_apply_reverb(&mut buffer, current_sample_rate, 0.5, 0.8, 0.15);
            }
            AudioEffect::ReverbSpatious => {
                post_apply_reverb(&mut buffer, current_sample_rate, 0.85, 0.5, 0.3);
            }
            AudioEffect::Spaceship => {
                post_apply_flanger(&mut buffer, current_sample_rate, 0.25, 1.0, 6.0, 0.8, 0.5);
            }
            AudioEffect::Underwater => {
                post_apply_phaser(
                    &mut buffer,
                    current_sample_rate,
                    1.5,
                    150.0,
                    1200.0,
                    0.6,
                    0.7,
                );
            }
            AudioEffect::PitchUp => {
                post_apply_pitch_shift(&mut buffer, current_sample_rate, 1.5, 1.0);
            }
            AudioEffect::PitchDown => {
                post_apply_pitch_shift(&mut buffer, current_sample_rate, 0.75, 1.0);
            }
            AudioEffect::Demon => {
                post_apply_pitch_shift(&mut buffer, current_sample_rate, 0.8, 0.65);
            }
            AudioEffect::Walkie => {
                post_apply_walkie_talkie(&mut buffer, current_sample_rate, 1200.0, 1.5, 5.0, 1.0);
            }
            AudioEffect::Popstar => {
                post_apply_popstar(&mut buffer, current_sample_rate, 0.02, 0.8);
            }
            AudioEffect::Robot => {
                post_apply_robot(&mut buffer, current_sample_rate, 55.0, 4.0);
            }
        }

        if let Some(amplify) = config.amplify {
            for channel in &mut buffer {
                for v in channel {
                    *v *= amplify;
                }
            }
        }

        // finally interleave the channels for the player
        let num_channels = config.target_format.channel_count.to_number() as usize;
        let total_frames = buffer[0].len();
        let num_frames = std::cmp::min(self.buff.producer.slots() / num_channels, total_frames);

        if num_frames > 0 {
            match self
                .buff
                .producer
                .write_chunk_uninit(num_frames * num_channels)
            {
                Ok(chunk) => {
                    let buffer_ref = &buffer;

                    chunk.fill_from_iter((0..num_frames).flat_map(|frame_idx| {
                        (0..num_channels).map(move |channel_idx| {
                            // reuse the last channel if the phone sends less channels
                            let channel = std::cmp::min(channel_idx, buffer_ref.len() - 1);
                            buffer_ref[channel].get(frame_idx).copied().unwrap_or(0.0)
                        })
                    }));
                    self.buff.commit(num_frames);
                }
                Err(e) => {
                    warn!("dropped audio samples {e}");
                }
            };
        }

        // the player keeps the latency in check, this only happens if it stopped reading
        if num_frames < total_frames {
            warn!("dropped {} audio frames", total_frames - num_frames);
        }

        let buffer_mono = if self.is_window_visible {
            // prepare mono channel buffer to return
            if config.target_format.channel_count.to_number() == 1 {
                Some(buffer[0].clone())
            } else {
                // if not mono, average the channels
                let mut mono_buffer: Vec<f32> = Vec::with_capacity(buffer[0].len());
                for i in 0..buffer[0].len() {
                    let sample: f32 = buffer.iter().map(|ch| ch[i]).sum::<f32>()
                        / config.target_format.channel_count.to_number() as f32;
                    mono_buffer.push(sample);
                }
                Some(mono_buffer)
            }
        } else {
            None
        };

        Ok(buffer_mono)
    }
}

pub fn convert_packet_to_f32(packet: &AudioPacketMessage) -> anyhow::Result<Vec<Vec<f32>>> {
    let audio_format = AudioFormat::from_android_format(packet.audio_format).unwrap();
    match audio_format {
        AudioFormat::U8 => convert_packet_to_f32_internal::<u8>(packet),
        AudioFormat::I16 => convert_packet_to_f32_internal::<i16>(packet),
        AudioFormat::I24 => convert_packet_to_f32_internal::<f32>(packet),
        AudioFormat::I32 => convert_packet_to_f32_internal::<i32>(packet),
        AudioFormat::F32 => convert_packet_to_f32_internal::<f32>(packet),
    }
}

fn convert_packet_to_f32_internal<F>(packet: &AudioPacketMessage) -> anyhow::Result<Vec<Vec<f32>>>
where
    F: cpal::SizedSample + AudioBytes + std::fmt::Debug + 'static,
{
    let audio_format: AudioFormat = AudioFormat::from_android_format(packet.audio_format).unwrap();
    let channel_count = packet.channel_count as usize;
    let samples_per_channel = packet.buffer.len() / (audio_format.sample_size() * channel_count);

    // Initialize a vector to hold the results for each channel
    let mut result = vec![Vec::with_capacity(samples_per_channel); channel_count];

    let sample_size = audio_format.sample_size();
    for frame in packet.buffer.chunks_exact(sample_size * channel_count) {
        for (ch, buf) in frame.chunks_exact(sample_size).enumerate() {
            result[ch].push(F::from_bytes(buf).to_f32());
        }
    }

    Ok(result)
}

fn convert_packet_to_f32_mono(packet: &AudioPacketMessage) -> anyhow::Result<Vec<f32>> {
    let audio_format = AudioFormat::from_android_format(packet.audio_format).unwrap();
    match audio_format {
        AudioFormat::U8 => convert_packet_to_f32_mono_internal::<u8>(packet),
        AudioFormat::I16 => convert_packet_to_f32_mono_internal::<i16>(packet),
        AudioFormat::I24 => convert_packet_to_f32_mono_internal::<f32>(packet),
        AudioFormat::I32 => convert_packet_to_f32_mono_internal::<i32>(packet),
        AudioFormat::F32 => convert_packet_to_f32_mono_internal::<f32>(packet),
    }
}

fn convert_packet_to_f32_mono_internal<F>(packet: &AudioPacketMessage) -> anyhow::Result<Vec<f32>>
where
    F: cpal::SizedSample + AudioBytes + std::fmt::Debug + 'static,
{
    let audio_format: AudioFormat = AudioFormat::from_android_format(packet.audio_format).unwrap();
    let channel_count = packet.channel_count as usize;

    let mut result = Vec::<f32>::with_capacity(
        packet.buffer.len() / (audio_format.sample_size() * channel_count),
    );

    for buf in packet
        .buffer
        .chunks_exact(audio_format.sample_size() * channel_count)
    {
        if channel_count == 1 {
            // For mono, there's just one sample
            result.push(F::from_bytes(buf).to_f32());
        } else {
            // For stereo, we merge the two samples into one
            let left = F::from_bytes(&buf[0..audio_format.sample_size()]).to_f32();
            let right = F::from_bytes(&buf[audio_format.sample_size()..]).to_f32();

            result.push((left + right) / 2.0); // Mix the two channels
        }
    }

    Ok(result)
}
