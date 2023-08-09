use std::{
    io::{self},
    net::{Ipv4Addr, UdpSocket},
    time::Duration,
};

use rtrb::{chunks::ChunkError, Producer};

use crate::streamer::{Streamer, WriteError, DEFAULT_PORT, IO_BUF_SIZE};

pub struct UdpStreamer {
    ip: Ipv4Addr,
    port: u16,
    socket: UdpSocket,
    producer: Producer<u8>,
    io_buf: [u8; 1024],
}

impl Streamer for UdpStreamer {
    fn new(shared_buf: Producer<u8>, ip: Ipv4Addr) -> Option<UdpStreamer> {
        let socket = if let Ok(socket) = UdpSocket::bind((ip, DEFAULT_PORT)) {
            socket
        } else {
            UdpSocket::bind((ip, 0)).expect("Failed to bind to socket")
        };

        socket
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();

        let addr = match socket.local_addr() {
            Ok(addr) => addr,
            Err(e) => {
                dbg!(e);
                return None;
            }
        };
        println!("UDP server listening on {}", addr);

        Some(Self {
            ip,
            port: addr.port(),
            socket,
            producer: shared_buf,
            io_buf: [0u8; IO_BUF_SIZE],
        })
    }

    fn process(&mut self) -> Result<usize, WriteError> {
        // Receive data into the buffer
        match self.socket.recv_from(&mut self.io_buf) {
            Ok((size, _)) => match self.producer.write_chunk_uninit(size) {
                Ok(chunk) => {
                    let moved_item = chunk.fill_from_iter(self.io_buf);
                    if moved_item == size {
                        Ok(size)
                    } else {
                        Err(WriteError::BufferOverfilled(moved_item, size - moved_item))
                    }
                }
                Err(ChunkError::TooFewSlots(remaining_slots)) => {
                    let chunk = self.producer.write_chunk_uninit(remaining_slots).unwrap();

                    let moved_item = chunk.fill_from_iter(self.io_buf);

                    Err(WriteError::BufferOverfilled(moved_item, size - moved_item))
                }
            },
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::TimedOut => Ok(0), // timeout use to check for input on stdin
                    io::ErrorKind::WouldBlock => Ok(0), // trigger on Linux when there is no stream input
                    _ => Err(WriteError::Io(e)),
                }
            }
        }
    }
}
