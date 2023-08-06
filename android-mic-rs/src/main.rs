use anyhow::{self};
use std::net::UdpSocket;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample,
};

fn main() -> anyhow::Result<()> {
    // Replace this with the port you want to bind to.
    let bind_port = 55555;

    // Create a UDP socket and bind it to the specified port
    let socket = UdpSocket::bind(("0.0.0.0", bind_port)).expect("Failed to bind to socket");

    // Buffer to store received data
    let mut buf = [0u8; 1024];

    println!("Waiting for data...");
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();
    println!("Default output config: {:?}", config);

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    loop {
        // Receive data into the buffer
        match socket.recv_from(&mut buf) {
            Ok((size, src_addr)) => {
                let data = &buf[..size];
                let src_addr = src_addr.to_string();
                println!("Received {} bytes from {}: {:?}", size, src_addr, data);
            }
            Err(e) => {
                eprintln!("Error while receiving data: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}






fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}