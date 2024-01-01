//! Feeds back the input stream directly into the output stream.
//!
//! Assumes that the input and output devices can use the same stream configuration and that they
//! support the f32 sample format.
//!
//! Uses a delay of `LATENCY_MS` milliseconds in case the default input and output streams are not
//! precisely synchronised.

use clap::Parser;
use cpal::traits::{ DeviceTrait, HostTrait, StreamTrait };
use ringbuf::HeapRb;
use audio_gate::NoiseGate;

#[derive(Parser, Debug)]
#[command(version, about = "CPAL feedback example", long_about = None)]
struct Opt {
    /// The input audio device to use
    #[arg(short, long, value_name = "IN", default_value_t = String::from("default"))]
    input_device: String,

    /// The output audio device to use
    #[arg(short, long, value_name = "OUT", default_value_t = String::from("default"))]
    output_device: String,

    /// Specify the delay between input and output
    #[arg(short, long, value_name = "DELAY_MS", default_value_t = 250.0)]
    latency: f32,

    #[arg(short, long)]
    with_gate: bool,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();
    let output_device = host.default_output_device().unwrap();

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = input_device.default_input_config()?.into();

    // Create a delay in case the input and output devices aren't synced.
    let latency_frames = (opt.latency / 1_000.0) * (config.sample_rate.0 as f32);
    let latency_samples = (latency_frames as usize) * (config.channels as usize);

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let mut gate = NoiseGate::new(-36.0, -54.0, 48000.0, 2, 150.0, 25.0, 150.0);
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        if opt.with_gate {
            let gated_data = gate.process_frame(&data);
            for &sample in gated_data.as_slice() {
                producer.push(sample).unwrap_or({});
            }
        } else {
            for &sample in data {
                producer.push(sample).unwrap_or({});
            }
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data {
            *sample = match consumer.pop() {
                Some(s) => s,
                None => { 0.0 }
            };
        }
    };

    // Build streams.
    println!("Attempting to build both streams with f32 samples and `{:?}`.", config);
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn, None)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        opt.latency
    );
    input_stream.play()?;
    output_stream.play()?;

    // Run for 3 seconds before closing.
    println!("Playing for 30 seconds... ");
    std::thread::sleep(std::time::Duration::from_secs(30));
    drop(input_stream);
    drop(output_stream);
    println!("Done!");
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
