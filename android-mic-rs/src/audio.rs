use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, Device, Host, SizedSample, StreamConfig,
};

use rtrb::{chunks::ReadChunkIntoIter, Consumer};

use crate::user_action::{Args, AudioFormat, ChannelCount};

pub fn setup_audio(consumer: Consumer<u8>, args: &Args) -> Result<cpal::Stream, BuildStreamError> {
    let host = cpal::default_host();

    print_output_devices(&host);

    let device_opt = match args.output_device {
        0 => host.default_output_device(),
        _ => {
            let ouput_devices = host.output_devices().unwrap();
            let mut device_opt = None;
            for (device_index, device) in ouput_devices.enumerate() {
                if args.output_device == device_index + 1 {
                    device_opt = Some(device)
                }
            }
            device_opt
        }
    };

    let Some(device) = device_opt else {
        eprintln!("Selected utput device not found.");
        return Err(BuildStreamError::DeviceNotAvailable);
    };

    let default_config: cpal::StreamConfig = device.default_output_config().unwrap().into();

    let channel_count = if let Some(channel_count) = &args.channel_count {
        match channel_count {
            ChannelCount::Mono => 1,
            ChannelCount::Stereo => 2,
        }
    } else {
        default_config.channels
    };

    let sample_rate = if let Some(sample_rate) = args.sample_rate {
        cpal::SampleRate(sample_rate)
    } else {
        default_config.sample_rate
    };

    let config = StreamConfig {
        channels: channel_count,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    println!();
    println!("Audio config:");
    println!("- number of channel: {}", config.channels);
    println!("- sample rate: {}", config.sample_rate.0);
    println!("- buffer size: {:?}", config.buffer_size);
    println!();

    match args.audio_format {
        AudioFormat::I16 => build::<i16>(&device, &config, consumer),
        AudioFormat::I32 => build::<i32>(&device, &config, consumer),
    }
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: Consumer<u8>,
) -> Result<cpal::Stream, BuildStreamError>
where
    T: Format + SizedSample,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channel_count = config.channels as usize;
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // data is the internal buf of cpal
            // we try to read the exact lenght of data from the shared buf here
            if let Ok(chunk) = consumer.read_chunk(data.len()) {
                // transform the part of sharred buf into an iter
                // only itered byte will be remove
                let mut iter = chunk.into_iter();

                // a frame contain sample * channel_count
                // a sample contain a value of type Format
                for frame in data.chunks_mut(channel_count) {
                    // the number of byte read to make a value depend on
                    // the format, so we pass an iter to produce the value
                    // - i16: 2 bytes
                    // - i32: 4 bytes
                    if let Some(value) = T::from_chunk(&mut iter) {
                        // sometime, a device will only support stereo
                        // but Android will record in mono
                        // so we have to clone the value in each channel in this case
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                }
            }
        },
        err_fn,
        None, // todo: try timeout
    )
}

trait Format {
    fn from_chunk(chunk: &mut ReadChunkIntoIter<'_, u8>) -> Option<Self>
    where
        Self: Sized;
}

impl Format for i16 {
    fn from_chunk(chunk: &mut ReadChunkIntoIter<'_, u8>) -> Option<Self> {
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
    fn from_chunk(chunk: &mut ReadChunkIntoIter<'_, u8>) -> Option<Self> {
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

fn print_output_devices(host: &Host) {
    println!("Default host: {}\n", host.id().name());

    let default_output: Option<Device> = host.default_output_device();

    if let Some(ref device) = default_output {
        println!("0. Default output device: {:?}\n", device.name().unwrap());
    };

    let ouput_devices = host.output_devices().unwrap();

    println!("All output devices:");
    for (device_index, device) in ouput_devices.enumerate() {
        println!("{}. \"{}\"", device_index + 1, device.name().unwrap());

        if let Ok(conf) = device.default_output_config() {
            println!(
                "    Default config: channel:{}, sample rate:{}, audio format:{}",
                conf.channels(),
                conf.sample_rate().0,
                conf.sample_format()
            );
        }
        let output_configs = match device.supported_output_configs() {
            Ok(f) => f.collect(),
            Err(e) => {
                println!("    Error getting supported output configs: {:?}", e);
                Vec::new()
            }
        };
        if !output_configs.is_empty() {
            println!("    Supported configs:");
            for (config_index, conf) in output_configs.into_iter().enumerate() {
                println!(
                    "      {}.{} channel:{}, min sample rate:{}, max sample rate:{}, audio format:{}",
                    device_index + 1,
                    config_index + 1,
                    conf.channels(),
                    conf.min_sample_rate().0,
                    conf.max_sample_rate().0,
                    conf.sample_format()
                );
            }
        }
    }
}
