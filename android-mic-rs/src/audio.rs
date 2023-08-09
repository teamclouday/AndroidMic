use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, Sample, SizedSample,
};

use rtrb::{chunks::{ChunkError, ReadChunkIntoIter}, Consumer};

use crate::user_action::Args;

pub fn setup_audio(
    consumer: Consumer<u8>,
    _args: &Args,
) -> Result<cpal::Stream, BuildStreamError> {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    let support = device.supported_output_configs().unwrap();

    for conf in support {

        dbg!(conf);

    }

    enumerate();
    

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

    let audio_format = AudioFormat::I16;

    match audio_format {
        AudioFormat::I16 => {
            build::<i16>(&device, &config, consumer, channels)
        },
        AudioFormat::I32 => {
            build::<i32>(&device, &config, consumer, channels)
        },
    }

    /*
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
                        //let result_i16: i16 = (byte2 as i16) << 8 | byte1 as i16;

                        // cursor method (should work on more PC but less optimize i think)
                        let mut cursor: Cursor<Vec<u8>> = Cursor::new(vec![byte1, byte2]);
                        let result_i16 = cursor.read_i16::<LittleEndian>().unwrap();
        

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
     */
}




fn build<T>(device: &cpal::Device, config: &cpal::StreamConfig, mut consumer: Consumer<u8>, channels: usize) -> Result<cpal::Stream, BuildStreamError>
where
    T: Format + SizedSample,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            
            match consumer.read_chunk(data.len()) {
                Ok(chunk) => {
                    let mut iter = chunk.into_iter();
                    for frame in data.chunks_mut(channels) {
                       
                        if let Some(value) = T::from_chunk(&mut iter) {
                            for sample in frame.iter_mut() {
                                *sample = value;
                            }
                        }
                    }
                }
                _ => {}
                
            }
        },
        err_fn,
        None, // todo: try timeout
    )
}


enum AudioFormat {
    I16,
    I32,
}

trait Format {
    fn from_chunk<'a>(chunk: &mut ReadChunkIntoIter<'a, u8>) -> Option<Self> where Self: Sized;
}



impl Format for i16 {
    fn from_chunk<'a>(chunk: &mut ReadChunkIntoIter<'a, u8>) -> Option<Self> {
        let Some(byte1) = chunk.next()  else {
            return None;
        };
        let Some(byte2) = chunk.next()  else {
            return None;
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(vec![byte1, byte2]);
        Some(cursor.read_i16::<LittleEndian>().unwrap())

    }
}

impl Format for i32 {
    fn from_chunk<'a>(chunk: &mut ReadChunkIntoIter<'a, u8>) -> Option<Self> {
        let Some(byte1) = chunk.next()  else {
            return None;
        };
        let Some(byte2) = chunk.next()  else {
            return None;
        };
        
        let Some(byte3) = chunk.next()  else {
            return None;
        };
        let Some(byte4) = chunk.next()  else {
            return None;
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(vec![byte1, byte2, byte3, byte4]);
        Some(cursor.read_i32::<LittleEndian>().unwrap())

    }
}











fn enumerate() {
    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    println!("Available hosts:\n  {:?}", available_hosts);

    for host_id in available_hosts {
        println!("{}", host_id.name());
        let host = cpal::host_from_id(host_id).unwrap();

        let default_in = host.default_input_device().map(|e| e.name().unwrap());
        let default_out = host.default_output_device().map(|e| e.name().unwrap());
        println!("  Default Input Device:\n    {:?}", default_in);
        println!("  Default Output Device:\n    {:?}", default_out);

        let devices = host.devices().unwrap();
        println!("  Devices: ");
        for (device_index, device) in devices.enumerate() {
            println!("  {}. \"{}\"", device_index + 1, device.name().unwrap());

            // Input configs
            if let Ok(conf) = device.default_input_config() {
                println!("    Default input stream config:\n      {:?}", conf);
            }
            let input_configs = match device.supported_input_configs() {
                Ok(f) => f.collect(),
                Err(e) => {
                    println!("    Error getting supported input configs: {:?}", e);
                    Vec::new()
                }
            };
            if !input_configs.is_empty() {
                println!("    All supported input stream configs:");
                for (config_index, config) in input_configs.into_iter().enumerate() {
                    println!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }

            // Output configs
            if let Ok(conf) = device.default_output_config() {
                println!("    Default output stream config:\n      {:?}", conf);
            }
            let output_configs = match device.supported_output_configs() {
                Ok(f) => f.collect(),
                Err(e) => {
                    println!("    Error getting supported output configs: {:?}", e);
                    Vec::new()
                }
            };
            if !output_configs.is_empty() {
                println!("    All supported output stream configs:");
                for (config_index, config) in output_configs.into_iter().enumerate() {
                    println!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }
        }
    }
}