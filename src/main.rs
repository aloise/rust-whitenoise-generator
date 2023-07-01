mod filters;

use std::time::Duration;
use std::{env, process, thread};
use rodio::{cpal, Device, DeviceTrait, OutputStream, source::Source};
use rand::{Rng};
use rodio::cpal::traits::HostTrait;
use filters::HighPassFilter;

// Our own source of white noise.
struct WhiteNoise {
    tick: usize,
    buffer: Vec<f32>,
    buffer_size: usize,
    ramp_up_samples: usize,
    sample_rate: u32,
}


impl WhiteNoise {
    fn new(sample_rate: u32, buffer_ms: usize, with_volume_ramp_up_ms: usize) -> Self {
        let buffer_samples = buffer_ms * sample_rate as usize / 1000;
        let ramp_up_samples = with_volume_ramp_up_ms * sample_rate as usize / 1000;

        let mut rng = rand::thread_rng();
        let cutoff_frequency = 60.0; // Hz
        let mut filter = HighPassFilter::new(sample_rate, cutoff_frequency);
        let mut noise_buffer: Vec<f32> = (0..buffer_samples).map(|_| {
            rng.gen::<f32>() * 2.0 - 1.0
        }).collect();
        filter.process_buffer(&mut noise_buffer);


        Self {
            tick: 0,
            buffer: noise_buffer.to_vec(),
            buffer_size: buffer_samples,
            ramp_up_samples,
            sample_rate
        }
    }
}

impl Iterator for WhiteNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let index: usize = self.tick % self.buffer_size;

        if self.ramp_up_samples > 0 && self.tick < self.ramp_up_samples {
            return Some(self.buffer[index] * self.tick as f32 / self.ramp_up_samples as f32);
        }

        self.tick += 1;

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
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}


fn play_noise_on_device(buffer_size_ms: usize, with_volume_ramp_up_ms: usize, dev: Device) {
    thread::spawn(move || {
        let device_name = dev.name().unwrap();

        let source = WhiteNoise::new(44100, buffer_size_ms, with_volume_ramp_up_ms);

        println!("Playing on device: {}", device_name);

        match OutputStream::try_from_device(&dev) {
            Ok((_stream, stream_handle)) => {
                stream_handle.play_raw(source).unwrap();
                thread::park();
            }
            Err(_) => eprintln!("Error creating stream on device {}", device_name),
        };
    });
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