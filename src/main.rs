use std::time::Duration;
use std::{env, process, thread};
use rodio::{OutputStream, source::Source};
use rand::random;

// Our own source of white noise.
struct WhiteNoise {
    elapsed: Duration,
    tick: usize,
    buffer: Vec<f32>,
    buffer_size: usize,
}

const WHITE_NOISE_SAMPLE_RATE: u32 = 44100;
const WHITE_NOISE_TICK_DURATION: Duration = Duration::from_micros(1_000_000 as u64 / WHITE_NOISE_SAMPLE_RATE as u64);

impl WhiteNoise {
    fn new(buffer_ms: usize) -> Self {
        let buffer_samples = buffer_ms * WHITE_NOISE_SAMPLE_RATE as usize / 1000;
        Self {
            elapsed: Duration::from_secs(0),
            tick: 0,
            buffer: (0..buffer_samples).map(|_| random::<f32>() * 2.0 - 1.0).collect(),
            buffer_size: buffer_samples,
        }
    }
}

impl Iterator for WhiteNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.elapsed += WHITE_NOISE_TICK_DURATION;
        self.tick = (self.tick + 1) % self.buffer_size;

        Some(self.buffer[self.tick])
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
    let num: usize = args.get(1)
        .map_or(10000, |arg| arg.parse().unwrap_or_else(|_| {
            eprintln!("Invalid argument. Exiting.");
            process::exit(1);
        }));

    println!("Playing White noise with a {}ms buffer", num);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let source = WhiteNoise::new(num);
    stream_handle.play_raw(source).unwrap();

    thread::park()
}