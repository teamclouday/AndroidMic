use std::{io, net::IpAddr, time::Duration};

use futures::StreamExt;
use prost::Message;
use tokio::net::UdpSocket;
use tokio_util::{codec::LengthDelimitedCodec, udp::UdpFramed};

use crate::{
    audio::process::ProcessCache,
    config::{AudioFormat, ConnectionMode},
    streamer::{
        AudioPacketMessage, CHECK_2, WriteError,
        message::{MessageWrapper, message_wrapper::Payload},
    },
};

use super::{AudioStream, ConnectError, StreamerMsg, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

/// a packet this far behind the expected one means the phone restarted its sequence
const SEQUENCE_RESTART_GAP: u32 = 100;
/// don't try to fill gaps bigger than this many packets with silence
const MAX_LOST_PACKETS: u32 = 50;

pub struct UdpStreamer {
    ip: IpAddr,
    pub port: u16,
    stream_config: AudioStream,
    framed: UdpFramed<LengthDelimitedCodec>,
    is_listening: bool,
    /// sequence number of the next packet we expect
    next_sequence: Option<u32>,
    process_cache: ProcessCache,
}

pub async fn new(
    ip: IpAddr,
    port: u16,
    stream_config: AudioStream,
) -> Result<UdpStreamer, ConnectError> {
    let socket = UdpSocket::bind((ip, port))
        .await
        .map_err(|e| ConnectError::CantBindPort(port, e))?;

    let addr = socket.local_addr().map_err(ConnectError::NoLocalAddress)?;

    let streamer = UdpStreamer {
        ip,
        port: addr.port(),
        stream_config,
        next_sequence: None,
        is_listening: true,
        framed: UdpFramed::new(socket, LengthDelimitedCodec::new()),
        process_cache: ProcessCache::new(),
    };

    Ok(streamer)
}

impl StreamerTrait for UdpStreamer {
    fn reconfigure_stream(&mut self, stream_config: AudioStream) {
        self.stream_config = stream_config;
        self.process_cache.clear();
    }

    fn status(&self) -> StreamerMsg {
        if self.is_listening {
            StreamerMsg::Listening {
                ip: Some(self.ip),
                port: Some(self.port),
            }
        } else {
            StreamerMsg::Connected {
                ip: Some(self.ip),
                port: Some(self.port),
                mode: ConnectionMode::Udp,
            }
        }
    }

    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        match tokio::time::timeout(
            Duration::from_secs(if self.is_listening {
                Duration::MAX.as_secs()
            } else {
                1
            }),
            self.framed.next(),
        )
        .await
        {
            Ok(res) => match res {
                Some(Ok((frame, addr))) => {
                    match MessageWrapper::decode(frame) {
                        Ok(packet) => {
                            match packet.payload {
                                Some(payload) => {
                                    let message = match payload {
                                        Payload::AudioPacket(packet) => 'packet: {
                                            let sequence = packet.sequence_number;
                                            let Some(packet) = packet.audio_packet else {
                                                break 'packet None;
                                            };

                                            if let Some(expected) = self.next_sequence {
                                                if sequence < expected
                                                    && expected - sequence < SEQUENCE_RESTART_GAP
                                                {
                                                    // its slot was already played, playing it
                                                    // now would shift everything after it
                                                    debug!(
                                                        "dropped late packet {sequence} (expected {expected})"
                                                    );
                                                    break 'packet None;
                                                }

                                                let lost = sequence.wrapping_sub(expected);
                                                if sequence > expected && lost <= MAX_LOST_PACKETS {
                                                    debug!("{lost} packets lost before {sequence}");
                                                    let frames = packet.buffer.len()
                                                        / (packet.channel_count.max(1) as usize
                                                            * AudioFormat::from_android_format(
                                                                packet.audio_format,
                                                            )
                                                            .map(|f| f.sample_size())
                                                            .unwrap_or(2));
                                                    self.stream_config.push_silence(
                                                        frames * lost as usize,
                                                        packet.sample_rate,
                                                    );
                                                }
                                            }
                                            self.next_sequence = Some(sequence.wrapping_add(1));

                                            let buffer_size = packet.buffer.len();
                                            let sample_rate = packet.sample_rate;

                                            match self.stream_config.process_audio_packet(
                                                packet,
                                                &mut self.process_cache,
                                            ) {
                                                Ok(Some(buffer)) => {
                                                    debug!(
                                                        "From {:?}, received {} bytes",
                                                        addr, buffer_size
                                                    );
                                                    Some(StreamerMsg::UpdateAudioWave {
                                                        data: AudioPacketMessage::to_wave_data(
                                                            &buffer,
                                                            sample_rate,
                                                        ),
                                                    })
                                                }
                                                _ => None,
                                            }
                                        }
                                        Payload::Connect(_) => {
                                            self.framed
                                                .get_ref()
                                                .send_to(CHECK_2.as_bytes(), &addr)
                                                .await
                                                .map_err(|e| {
                                                    ConnectError::HandShakeFailed("writing", e)
                                                })?;

                                            None
                                        }
                                    };

                                    if self.is_listening {
                                        self.is_listening = false;
                                        Ok(Some(StreamerMsg::Connected {
                                            ip: Some(self.ip),
                                            port: Some(self.port),
                                            mode: ConnectionMode::Udp,
                                        }))
                                    } else {
                                        Ok(message)
                                    }
                                }
                                None => todo!(),
                            }
                        }
                        Err(e) => Err(ConnectError::WriteError(WriteError::Deserializer(e))),
                    }
                }

                Some(Err(e)) => {
                    match e.kind() {
                        io::ErrorKind::TimedOut => Ok(None), // timeout use to check for input on stdin
                        io::ErrorKind::WouldBlock => Ok(None), // trigger on Linux when there is no stream input
                        _ => Err(WriteError::Io(e))?,
                    }
                }
                None => {
                    todo!()
                }
            },
            Err(_) => {
                self.is_listening = true;
                self.next_sequence = None;
                Ok(Some(StreamerMsg::Listening {
                    ip: Some(self.ip),
                    port: Some(self.port),
                }))
            }
        }
    }
}
