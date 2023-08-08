use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
    time::Duration,
};

use rtrb::{chunks::ChunkError, Producer};

use crate::streamer::{
    Streamer, WriteError, DEFAULT_PORT, DEVICE_CHECK, DEVICE_CHECK_EXPECTED, IO_BUF_SIZE,
};

// process pour la version C#

// choisir un adress ip parmis les interfaces reseau de l'ordinateur
// creer un listener
// essaye de bind un port dessus
// call BeginAccept (asynchrome with call back)
// in the call back: fait un check de transmition de data
// stream manipulation

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);
pub struct TcpStreamer {
    ip: String,
    port: u16,
    stream: TcpStream,
    producer: Producer<u8>,
    io_buf: [u8; 1024],
}

impl Streamer for TcpStreamer {
    fn new(shared_buf: Producer<u8>, ip: String) -> Option<Self> {
        let listener = if let Ok(listener) = TcpListener::bind((ip.clone(), DEFAULT_PORT)) {
            listener
        } else {
            TcpListener::bind((ip.clone(), 0)).expect("can't bind listener")
        };

        let addr = match TcpListener::local_addr(&listener) {
            Ok(addr) => addr,
            Err(e) => {
                dbg!(e);
                return None;
            }
        };
        println!("TCP server listening on {}", addr);

        match listener.accept() {
            Ok((mut stream, addr)) => {
                if let Err(e) = stream.set_read_timeout(Some(MAX_WAIT_TIME)) {
                    eprintln!("can't set read time out: {}", e);
                }
                if let Err(e) = stream.set_write_timeout(Some(MAX_WAIT_TIME)) {
                    eprintln!("can't set write time out: {}", e);
                }

                // read check
                let mut check_buf = [0u8; DEVICE_CHECK_EXPECTED.len()];
                // read_to_string doesn't works somehow, we need fix buffer
                match stream.read(&mut check_buf) {
                    Ok(_) => {
                        let message = from_utf8(&check_buf).unwrap();
                        if DEVICE_CHECK_EXPECTED != message {
                            println!(
                                "read check fail: expected = {:x?}, received = {:x?}",
                                DEVICE_CHECK_EXPECTED, message
                            );
                            return None;
                        }
                    }
                    Err(e) => {
                        println!("read check fail: {:?}", e);
                        return None;
                    }
                }

                // write check
                if let Err(e) = stream.write(DEVICE_CHECK.as_bytes()) {
                    println!("write check fail: {:?}", e);
                    return None;
                }

                println!("connection accepted, address: {}", addr);

                Some(Self {
                    ip,
                    port: addr.port(),
                    stream,
                    producer: shared_buf,
                    io_buf: [0u8; IO_BUF_SIZE],
                })
            }
            Err(e) => {
                dbg!(e);
                None
            }
        }
    }

    fn process(&mut self) -> Result<usize, WriteError> {
        match self.stream.read(&mut self.io_buf) {
            Ok(size) => match self.producer.write_chunk_uninit(size) {
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
