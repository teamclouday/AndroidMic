use byteordered::byteorder::{self, ByteOrder};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, Device, Host, SizedSample, StreamConfig,
};

use rtrb::{
    chunks::{ChunkError, ReadChunkIntoIter},
    Consumer,
};

use crate::config::{AudioFormat, ChannelCount};


pub fn setup_audio<E: ByteOrder>(
    consumer: Consumer<u8>,
    args: &Args,
) -> Result<cpal::Stream, BuildStreamError> {

    let host = cpal::default_host();

    print_output_devices(&host, args);

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
        eprintln!("Selected output device not found.");
        return Err(BuildStreamError::DeviceNotAvailable);
    };

    let default_config: cpal::StreamConfig = device.default_output_config().unwrap().into();

    let mut channel_count = if let Some(channel_count) = &args.channel_count {
        match channel_count {
            ChannelCount::Mono => 1,
            ChannelCount::Stereo => 2,
        }
    } else {
        default_config.channels
    };

    let channel_strategy = match ChannelStrategy::new(&device, channel_count) {
        Some(strategy) => {
            if strategy == ChannelStrategy::MonoCloned {
                // notify the user because it will change the printed config
                println!("Only stereo is supported, fall back to mono cloned strategy.");
                channel_count = 2;
            }
            strategy
        }
        None => {
            eprintln!("unsupported channels configuration.");
            return Err(BuildStreamError::StreamConfigNotSupported);
        }
    };

    let sample_rate = if let Some(sample_rate) = args.sample_rate {
        cpal::SampleRate(sample_rate)
    } else {
        default_config.sample_rate
    };

    let config = StreamConfig {
        channels: channel_count,
        sample_rate,
        buffer_size: default_config.buffer_size,
    };

    println!();
    println!("Audio config:");
    println!("- selected device: {}", args.output_device);
    println!("- number of channel: {}", config.channels);
    println!("- sample rate: {}", config.sample_rate.0);
    println!("- buffer size: {:?}", config.buffer_size);
    println!("- audio format: {}", args.audio_format);
    println!();

    match args.audio_format {
        AudioFormat::I8 => todo!(),
        AudioFormat::I16 => build::<i16, E>(&device, &config, consumer, channel_strategy),
        AudioFormat::I24 => todo!(),
        AudioFormat::I32 => build::<i32, E>(&device, &config, consumer, channel_strategy),
        AudioFormat::I48 => todo!(),
        AudioFormat::I64 => todo!(),
        AudioFormat::U8 => todo!(),
        AudioFormat::U16 => todo!(),
        AudioFormat::U24 => todo!(),
        AudioFormat::U32 => todo!(),
        AudioFormat::U48 => todo!(),
        AudioFormat::U64 => todo!(),
        AudioFormat::F32 => build::<f32, E>(&device, &config, consumer, channel_strategy),
        AudioFormat::F64 => todo!(),
    }
}

fn build<F, E>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: Consumer<u8>,
    channel_strategy: ChannelStrategy,
) -> Result<cpal::Stream, BuildStreamError>
where
    F: Format + SizedSample,
    E: ByteOrder,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channel_count = config.channels as usize;
    device.build_output_stream(
        config,
        move |data: &mut [F], _: &cpal::OutputCallbackInfo| {
            // data is the internal buf of cpal
            // we try to read the exact lenght of data from the shared buf here
            match consumer.read_chunk(data.len()) {
                Ok(chunk) => {
                    // transform the part of sharred buf into an iter
                    // only itered byte will be remove
                    let mut chunk_iter = chunk.into_iter();

                    // a frame contain sample * channel_count
                    // a sample contain a value of type Format
                    for frame in data.chunks_mut(channel_count) {
                        channel_strategy.fill_frame::<F, E>(frame, &mut chunk_iter);
                    }
                }
                // fallback
                Err(ChunkError::TooFewSlots(available_slots)) => {
                    let mut chunk_iter = consumer.read_chunk(available_slots).unwrap().into_iter();
                    for frame in data.chunks_mut(channel_count) {
                        channel_strategy.fill_frame::<F, E>(frame, &mut chunk_iter);
                    }
                }
            }
        },
        err_fn,
        None, // todo: find out what this does
    )
}

trait Format {
    // the number of byte read to make a value depend on
    // the format, so we pass an iter to produce the value
    // - i16: 2 bytes
    // - i32: 4 bytes
    fn produce_value_from_chunk<E>(chunk: &mut ReadChunkIntoIter<'_, u8>) -> Option<Self>
    where
        Self: Sized,
        E: ByteOrder;
}

impl Format for i16 {
    fn produce_value_from_chunk<T: ByteOrder>(
        chunk: &mut ReadChunkIntoIter<'_, u8>,
    ) -> Option<Self> {
        let Some(byte1) = chunk.next() else {
            return None;
        };
        let Some(byte2) = chunk.next() else {
            return None;
        };
        Some(T::read_i16(&[byte1, byte2]))
    }
}

// not tested
impl Format for i32 {
    fn produce_value_from_chunk<T: byteorder::ByteOrder>(
        chunk: &mut ReadChunkIntoIter<'_, u8>,
    ) -> Option<Self> {
        let Some(byte1) = chunk.next() else {
            return None;
        };
        let Some(byte2) = chunk.next() else {
            return None;
        };

        let Some(byte3) = chunk.next() else {
            return None;
        };
        let Some(byte4) = chunk.next() else {
            return None;
        };

        Some(T::read_i32(&[byte1, byte2, byte3, byte4]))
    }
}

// not tested
impl Format for f32 {
    fn produce_value_from_chunk<T: byteorder::ByteOrder>(
        chunk: &mut ReadChunkIntoIter<'_, u8>,
    ) -> Option<Self> {
        let Some(byte1) = chunk.next() else {
            return None;
        };
        let Some(byte2) = chunk.next() else {
            return None;
        };

        let Some(byte3) = chunk.next() else {
            return None;
        };
        let Some(byte4) = chunk.next() else {
            return None;
        };

        Some(T::read_f32(&[byte1, byte2, byte3, byte4]))
    }
}

fn print_output_devices(host: &Host, args: &Args) {
    println!("Default host: {}\n", host.id().name());

    let default_output: Option<Device> = host.default_output_device();

    if let Some(ref device) = default_output {
        println!(
            "Default output device:\n    0. {:?}\n",
            device.name().unwrap()
        );
    };

    let ouput_devices = host.output_devices().unwrap();

    println!("All output devices:");
    for (device_index, device) in ouput_devices.enumerate() {
        println!("    {}. \"{}\"", device_index + 1, device.name().unwrap());

        if let Ok(conf) = device.default_output_config() {
            println!(
                "        Default config: channel:{}, sample rate:{}, audio format:{}",
                conf.channels(),
                conf.sample_rate().0,
                conf.sample_format()
            );
        }
        let output_configs = match device.supported_output_configs() {
            Ok(f) => f.collect(),
            Err(e) => {
                println!("        Error getting supported output configs: {:?}", e);
                Vec::new()
            }
        };
        if args.show_supported_audio_config && !output_configs.is_empty() {
            println!("        Supported configs:");
            for (config_index, conf) in output_configs.into_iter().enumerate() {
                println!(
                    "            {}.{} channel:{}, min sample rate:{}, max sample rate:{}, audio format:{}",
                    device_index + 1,
                    config_index + 1,
                    conf.channels(),
                    conf.min_sample_rate().0,
                    conf.max_sample_rate().0,
                    conf.sample_format()
                );
            }
        }
        println!();
    }
}

#[derive(PartialEq)]
enum ChannelStrategy {
    Mono,
    Stereo,
    MonoCloned,
}

impl ChannelStrategy {
    /// in: Stereo / out: Mono -> None
    /// in: Mono / out: Stero -> MonoCloned
    /// in: Mono / out: Mono -> Mono
    /// in: Stereo / out: Stero -> Stereo
    fn new(device: &Device, channel_count: u16) -> Option<ChannelStrategy> {
        let supported_channels = device
            .supported_output_configs()
            .unwrap()
            .map(|config| config.channels());

        let mut fall_back = None;

        for supported_channel in supported_channels {
            if supported_channel == channel_count {
                match channel_count {
                    1 => return Some(ChannelStrategy::Mono),
                    2 => return Some(ChannelStrategy::Stereo),
                    _ => {}
                }
            }
            if supported_channel == 2 && channel_count == 1 {
                fall_back = Some(ChannelStrategy::MonoCloned);
            }
        }

        fall_back
    }

    fn fill_frame<F, E>(&self, frame: &mut [F], chunk: &mut ReadChunkIntoIter<'_, u8>)
    where
        F: Format + SizedSample,
        E: ByteOrder,
    {
        match self {
            ChannelStrategy::Mono => {
                if let Some(value) = F::produce_value_from_chunk::<E>(chunk) {
                    frame[0] = value;
                }
            }
            ChannelStrategy::Stereo => {
                if let Some(value) = F::produce_value_from_chunk::<E>(chunk) {
                    frame[0] = value;
                }
                if let Some(value) = F::produce_value_from_chunk::<E>(chunk) {
                    frame[1] = value;
                }
            }
            ChannelStrategy::MonoCloned => {
                if let Some(value) = F::produce_value_from_chunk::<E>(chunk) {
                    frame[0] = value;
                    frame[1] = value;
                }
            }
        }
    }
}
