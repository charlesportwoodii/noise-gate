<div align="center">

  <h1>Noise Gate</h1>

    <a href="https://www.erianna.com"><img src="https://raw.githubusercontent.com/charlesportwoodii/noise-gate/master/logo.png" width="140"/></a>

  <p>
    <strong>A Simple Noise Gate for Rust</strong>
  </p>
  <hr />
</div>

## Noise Gate

A simple [Noise Gate](https://en.wikipedia.org/wiki/Noise_gate) algorithm written for Rust for handling of streaming audio from [CPAL](https://github.com/RustAudio/cpal). Noise Gates are extremely useful for cleaning up muddy audio streams: breathing noises, page turns, microphone knocks, keyboard clatter, HVAC hum, fan noises, room reverb, and so forth to help remove unwanted sounds from your stream. While a noise gate won't completely eliminate background noise, it can help make your audio stream sounds much cleaner.

## Usage

This crate is intended to be used with raw audio streams from CPAL or the like.

1. Add the crate to your project

```
cargo add audio-gate
```

2. Create the Noise Gate outside of your main stream (preferrably your input stream) with the desired parameters.

```rust
let mut gate = NoiseGate::new(
    -36.0, // Open Threshold
    -54.0, // Close Treshold
    48000.0, // Sample Rate
    2, // Channels
    150.0, // Release Rate
    25.0, // Attack Rate
    150.0 // Hold time
);
```

3. Process each frame of audio with the gate to remove unwanted sound. The following example is from CPAL

```rust
let stream = match device.build_input_stream(
    &config,
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let gated_data = gate.process_frame(&data);
        for &sample in gated_data.as_slice() {
            producer.push(sample).unwrap_or({});
        }
    },
    move |err| {},
    None
) {
    Ok(stream) => stream,
    Err(e) => {
        return Err(anyhow!("{}", e.to_string()));
    }
};
```

## Hear the difference

A feedback example is provided in the examples directory so that you can hear the difference a noise gate might apply to your audio stream.

To hear your current input with the noise gate applied run the example:

```
cargo run --example feedback -- --with-gate
```

To hear your audio with it not applied run the example without the flag:

```
cargo run --example feedback
```

## License

This application is licensed under the BSD 3 Clause License
