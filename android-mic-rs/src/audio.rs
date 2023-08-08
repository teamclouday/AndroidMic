use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError,
};

use rtrb::{chunks::ChunkError, Consumer};

pub fn setup_audio(mut consumer: Consumer<u8>) -> Result<cpal::Stream, BuildStreamError> {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    // let config = StreamConfig{
    //     channels: 1,
    //     sample_rate: SampleRate(16000),
    //     buffer_size: BufferSize::Default(1024),
    // };
    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();

    println!();
    println!("Audio config:");
    println!("- number of channel: {}", config.channels);
    println!("- sample rate: {}", config.sample_rate.0);
    println!("- buffer size: {:?}", config.buffer_size);
    println!();

    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            match consumer.read_chunk(data.len()) {
                Ok(chunk) => {
                    //println!("read_chunk {}", chunk.len());
                    let mut iter = chunk.into_iter();
                    // a frame has 480 samples
                    for frame in data.chunks_mut(channels) {
                        let Some(byte1) = iter.next()  else {
                            // should not happend, because we read data.len()
                            // chunk, but happend sometime
                            //eprintln!("None next byte1");
                            return;
                        };
                        let Some(byte2) = iter.next()  else {
                            //eprintln!("None next byte2, loose byte1");
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
                Err(ChunkError::TooFewSlots(available_slots)) => {
                    let mut iter = consumer.read_chunk(available_slots).unwrap().into_iter();
                    for frame in data.chunks_mut(channels) {
                        let Some(byte1) = iter.next()  else {
                            //eprintln!("None next byte1");
                            return;
                        };
                        let Some(byte2) = iter.next()  else {
                            //eprintln!("None next byte2, loose byte1");
                            return;
                        };
                        let result_i16: i16 = (byte2 as i16) << 8 | byte1 as i16;
                        for sample in frame.iter_mut() {
                            *sample = result_i16;
                        }
                    }
                }
            }
        },
        err_fn,
        None, // todo: try timeout
    )
}
