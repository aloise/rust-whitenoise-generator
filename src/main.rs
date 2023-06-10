
use std::time::Duration;
use std::io::BufReader;
use std::thread;
use rodio::{Decoder, OutputStream, source::Source, Sample};
use rand::random;

// Our own source of white noise.
struct WhiteNoise {
    duration: Duration,
    elapsed: Duration,
}

impl WhiteNoise {
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::from_secs(0),
        }
    }
}

impl Iterator for WhiteNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.elapsed >= self.duration {
            None
        } else {
            self.elapsed += Duration::from_secs_f32(1.0 / 44100.0);
            Some(random::<f32>() * 2.0 - 1.0)
        }
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
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() {
    print!("Playing White noise...");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let source = WhiteNoise::new(Duration::from_secs(5));
    stream_handle.play_raw(source).unwrap();

    thread::park()
}