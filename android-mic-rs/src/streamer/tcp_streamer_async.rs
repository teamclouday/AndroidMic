use std::{io, net::IpAddr, str::from_utf8, time::Duration};

use rtrb::{chunks::ChunkError, Producer};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::streamer::streamer_trait::{
    DEFAULT_PORT, DEVICE_CHECK, DEVICE_CHECK_EXPECTED, IO_BUF_SIZE, MAX_PORT,
};

use super::streamer_trait::{ConnectError, StreamerTrait, WriteError};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

pub struct TcpStreamer {
    ip: IpAddr,
    pub port: u16,
    stream: TcpStream,
    io_buf: [u8; 1024],
    disconnect_loop_detecter: u32,
}

pub async fn new(ip: IpAddr) -> Result<TcpStreamer, ConnectError> {
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

    info!("TCP server listening on {}", addr);

    let (mut stream, addr) = listener.accept().await.map_err(ConnectError::CantAccept)?;

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

    Ok(TcpStreamer {
        ip,
        port: addr.port(),
        stream,
        io_buf: [0u8; IO_BUF_SIZE],
        disconnect_loop_detecter: 0,
    })
}

impl StreamerTrait for TcpStreamer {
    async fn process(&mut self, producer: &mut Producer<u8>) -> Result<usize, WriteError> {
        match self.stream.read(&mut self.io_buf).await {
            Ok(size) => {
                if size == 0 {
                    if self.disconnect_loop_detecter >= DISCONNECT_LOOP_DETECTER_MAX {
                        return Err(WriteError::Io(io::Error::new(
                            io::ErrorKind::NotConnected,
                            "disconnect loop detected",
                        )));
                    } else {
                        self.disconnect_loop_detecter += 1
                    }
                } else {
                    self.disconnect_loop_detecter = 0;
                };
                match producer.write_chunk_uninit(size) {
                    Ok(chunk) => {
                        let moved_item = chunk.fill_from_iter(self.io_buf);
                        if moved_item == size {
                            Ok(size)
                        } else {
                            Err(WriteError::BufferOverfilled(moved_item, size - moved_item))
                        }
                    }
                    Err(ChunkError::TooFewSlots(remaining_slots)) => {
                        let chunk = producer.write_chunk_uninit(remaining_slots).unwrap();

                        let moved_item = chunk.fill_from_iter(self.io_buf);

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

    fn port(&self) -> Option<u16> {
        Some(self.port)
    }
}
