use anyhow::{self};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::RingBuffer;
use std::net::UdpSocket;

fn main() -> anyhow::Result<()> {
    // Replace this with the port you want to bind to.
    let bind_port = 55555;

    // Create a UDP socket and bind it to the specified port
    let socket = UdpSocket::bind(("0.0.0.0", bind_port)).expect("Failed to bind to socket");

    println!("Waiting for data...");
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    // let config = StreamConfig{
    //     channels: 1,
    //     sample_rate: SampleRate(16000),
    //     buffer_size: BufferSize::Default(1024),
    // };
    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();

    println!("Default output config: {:?}", config);

    let channels = config.channels as usize;

    // Buffer to store received data
    let capacity = 5 * 1024;
    let (mut producer, mut consumer) = RingBuffer::<u8>::new(capacity);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            match consumer.read_chunk(data.len()) {
                Ok(chunk) => {
                    println!("read_chunk {}", chunk.len());
                    let mut iter = chunk.into_iter();
                    // a frame has 480 samples
                    for frame in data.chunks_mut(channels) {
                        let Some(byte1) = iter.next()  else {
                            return;
                        };
                        let Some(byte2) = iter.next()  else {
                            return;
                        };

                        // Combine the two u8 values into a single i16
                        // don't ask me why we inverse bytes here (probably because of Endian stuff)
                        let result_i16: i16 = (byte2 as i16) << 8 | byte1 as i16;

                        // cursor method (should work on more PC but less optimize i think)
                        // let mut cursor: Cursor<Vec<u8>> = Cursor::new(vec![byte1, byte2]);
                        // let result_i16 = cursor.read_i16::<LittleEndian>().unwrap();

                        // a sample has two cases (probably left/right)
                        for sample in frame.iter_mut() {
                            *sample = result_i16;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error read_chunk {:?}", e);
                }
            }
        },
        err_fn,
        None, // todo: try timeout
    )?;
    stream.play()?;

    loop {
        let mut tmp_buf = [0u8; 1024];
        // Receive data into the buffer
        match socket.recv_from(&mut tmp_buf) {
            Ok((size, src_addr)) => {
                match producer.write_chunk_uninit(size) {
                    Ok(chunk) => {
                        chunk.fill_from_iter(tmp_buf);
                    }
                    Err(e) => {
                        eprintln!("Error write_chunk_uninit {:?}", e);
                        continue;
                    }
                }

                let src_addr = src_addr.to_string();
                println!("Received {} bytes from {}", size, src_addr);
            }
            Err(e) => {
                eprintln!("Error while receiving data: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}
