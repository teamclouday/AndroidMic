use std::{io, net::IpAddr, str::from_utf8, time::Duration};

use rtrb::{chunks::ChunkError, Producer};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::streamer::{
    WriteError, DEFAULT_PORT, DEVICE_CHECK, DEVICE_CHECK_EXPECTED, IO_BUF_SIZE, MAX_PORT,
};

use super::{ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;  

pub struct TcpStreamer {
    ip: IpAddr,
    pub port: u16,
    producer: Producer<u8>,
    state: TcpStreamerState,
}

#[allow(clippy::large_enum_variant)]
pub enum TcpStreamerState {
    Listening {
        listener: TcpListener,
    },
    Streaming {
        stream: TcpStream,
        io_buf: [u8; 1024],
        disconnect_loop_detecter: u32,
    },
}

pub async fn new(ip: IpAddr, producer: Producer<u8>) -> Result<TcpStreamer, ConnectError> {
    let mut listener = None;

    // try to always bind the same port, to not change it everytime Android side
    for p in DEFAULT_PORT..=MAX_PORT {
        if let Ok(l) = TcpListener::bind((ip, p)).await {
            listener = Some(l);
            break;
        }
    }

    let listener = if let Some(listener) = listener {
        listener
    } else {
        TcpListener::bind((ip, 0))
            .await
            .map_err(ConnectError::CantBindPort)?
    };

    let addr = TcpListener::local_addr(&listener).map_err(ConnectError::NoLocalAddress)?;

    let streamer = TcpStreamer {
        ip,
        port: addr.port(),
        producer,
        state: TcpStreamerState::Listening { listener },
    };

    Ok(streamer)
}

impl StreamerTrait for TcpStreamer {
    fn set_buff(&mut self, producer: Producer<u8>) {
        self.producer = producer;
    }

    fn status(&self) -> Option<Status> {
        match &self.state {
            TcpStreamerState::Listening { .. } => Some(Status::Listening {
                port: Some(self.port),
            }),
            TcpStreamerState::Streaming { .. } => Some(Status::Connected),
        }
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        match &mut self.state {
            TcpStreamerState::Listening { listener } => {
                let addr =
                    TcpListener::local_addr(listener).map_err(ConnectError::NoLocalAddress)?;

                info!("TCP server listening on {}", addr);

                let (mut stream, addr) =
                    listener.accept().await.map_err(ConnectError::CantAccept)?;

                // read check
                let mut check_buf = [0u8; DEVICE_CHECK_EXPECTED.len()];
                // read_to_string doesn't works somehow, we need a fixed buffer
                match stream.read(&mut check_buf).await {
                    Ok(_) => {
                        let message = from_utf8(&check_buf).unwrap();
                        if DEVICE_CHECK_EXPECTED != message {
                            return Err(ConnectError::CheckFailed {
                                expected: DEVICE_CHECK_EXPECTED,
                                received: message.into(),
                            });
                        }
                    }
                    Err(e) => {
                        return Err(ConnectError::CheckFailedIo(e));
                    }
                }

                // write check
                if let Err(e) = stream.write(DEVICE_CHECK.as_bytes()).await {
                    return Err(ConnectError::CheckFailedIo(e));
                }

                info!("connection accepted, remote address: {}", addr);

                self.state = TcpStreamerState::Streaming {
                    stream,
                    io_buf: [0u8; IO_BUF_SIZE],
                    disconnect_loop_detecter: 0,
                };

                Ok(Some(Status::Connected))
            }
            TcpStreamerState::Streaming {
                stream,
                io_buf,
                disconnect_loop_detecter,
            } => {
                async fn process(
                    stream: &mut TcpStream,
                    mut io_buf: [u8; 1024],
                    disconnect_loop_detecter: &mut u32,
                    producer: &mut Producer<u8>,
                ) -> Result<usize, WriteError> {
                    match stream.read(&mut io_buf).await {
                        Ok(size) => {
                            if size == 0 {
                                if *disconnect_loop_detecter >= DISCONNECT_LOOP_DETECTER_MAX {
                                    return Err(WriteError::Io(io::Error::new(
                                        io::ErrorKind::NotConnected,
                                        "disconnect loop detected",
                                    )));
                                } else {
                                    *disconnect_loop_detecter += 1
                                }
                            } else {
                                *disconnect_loop_detecter = 0;
                            };
                            match producer.write_chunk_uninit(size) {
                                Ok(chunk) => {
                                    let moved_item = chunk.fill_from_iter(io_buf);
                                    if moved_item == size {
                                        Ok(size)
                                    } else {
                                        Err(WriteError::BufferOverfilled(
                                            moved_item,
                                            size - moved_item,
                                        ))
                                    }
                                }
                                Err(ChunkError::TooFewSlots(remaining_slots)) => {
                                    let chunk =
                                        producer.write_chunk_uninit(remaining_slots).unwrap();

                                    let moved_item = chunk.fill_from_iter(io_buf);

                                    Err(WriteError::BufferOverfilled(moved_item, size - moved_item))
                                }
                            }
                        }
                        Err(e) => {
                            match e.kind() {
                                io::ErrorKind::TimedOut => Ok(0), // timeout use to check for input on stdin
                                io::ErrorKind::WouldBlock => Ok(0), // trigger on Linux when there is no stream input
                                _ => Err(WriteError::Io(e)),
                            }
                        }
                    }
                }

                match process(
                    stream,
                    *io_buf,
                    disconnect_loop_detecter,
                    &mut self.producer,
                )
                .await
                {
                    Ok(_moved) => Ok(None),
                    Err(e) => Err(ConnectError::WriteError(e)),
                }
            }
        }
    }
}
