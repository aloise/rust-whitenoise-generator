use std::time::Duration;
use std::{env, process, thread};
use rodio::{cpal, Device, DeviceTrait, OutputStream, source::Source};
use rand::random;
use rodio::cpal::traits::HostTrait;

// Our own source of white noise.
struct WhiteNoise {
    elapsed: Duration,
    tick: usize,
    buffer: Vec<f32>,
    buffer_size: usize,
    ramp_up_samples: usize,
}

const WHITE_NOISE_SAMPLE_RATE: u32 = 44100;
const WHITE_NOISE_TICK_DURATION: Duration = Duration::from_micros(1_000_000 as u64 / WHITE_NOISE_SAMPLE_RATE as u64);

impl WhiteNoise {
    fn new(buffer_ms: usize, with_volume_ramp_up_ms: usize) -> Self {
        let buffer_samples = buffer_ms * WHITE_NOISE_SAMPLE_RATE as usize / 1000;
        let ramp_up_samples = with_volume_ramp_up_ms * WHITE_NOISE_SAMPLE_RATE as usize / 1000;
        Self {
            elapsed: Duration::from_secs(0),
            tick: 0,
            buffer: (0..buffer_samples).map(|_| {
                random::<f32>() - 2.0
            }).collect(),
            buffer_size: buffer_samples,
            ramp_up_samples,
        }
    }
}

impl Iterator for WhiteNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.elapsed += WHITE_NOISE_TICK_DURATION;
        self.tick += 1;

        let index: usize = self.tick % self.buffer_size;

        if self.ramp_up_samples > 0 && self.tick < self.ramp_up_samples {
            return Some(self.buffer[index] * self.tick as f32 / self.ramp_up_samples as f32);
        }

        Some(self.buffer[index])
    }
}

impl Source for WhiteNoise {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        WHITE_NOISE_SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Try to parse the argument as a u16
    let buffer_size_ms: usize = args.get(1)
        .map_or(30000, |arg| arg.parse().unwrap_or_else(|_| {
            eprintln!("Invalid argument. Exiting.");
            process::exit(1);
        }));

    let with_volume_ramp_up_ms = 10000;
    println!("Playing White noise with a {}ms buffer", buffer_size_ms);

    let devices = match cpal::default_host().output_devices() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Invalid argument. Exiting.");
            process::exit(2)
        }
    };

    for dev in devices {
        play_noise_on_device(buffer_size_ms, with_volume_ramp_up_ms, dev);
    }

    thread::park()
}

fn play_noise_on_device(buffer_size_ms: usize, with_volume_ramp_up_ms: usize, dev: Device) {
    thread::spawn(move || {
        let device_name = dev.name().unwrap();

        let source = WhiteNoise::new(buffer_size_ms, with_volume_ramp_up_ms);

        println!("Playing on device: {}", device_name);

        match OutputStream::try_from_device(&dev) {
            Ok((_stream, stream_handle)) => {
                stream_handle.play_raw(source).unwrap();
                thread::park();
            },
            Err(_) => eprintln!("Error creating stream on device {}", device_name),
        };
    });
}