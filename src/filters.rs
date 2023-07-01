pub struct HighPassFilter {
    prev_input: f32,
    prev_output: f32,
    alpha: f32,
}


impl HighPassFilter {
    pub fn new(sample_rate: u32, cutoff_frequency: f32) -> Self {
        let dt = 1.0 / sample_rate as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_frequency);
        let alpha = rc / (rc + dt);

        Self {
            prev_input: 0.0,
            prev_output: 0.0,
            alpha,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output = output;
        output
    }

    pub fn process_buffer(&mut self, buffer: &mut Vec<f32>) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highpass_filter() {
        let sample_rate = 44100;
        let cutoff_frequency = 100.0;

        let mut buffer = vec![0.0; 10000];
        // Add some low frequency content to the buffer
        for (i, sample) in buffer.iter_mut().enumerate() {
            *sample += (i as f32 / sample_rate as f32 * 2.0 * std::f32::consts::PI * 50.0).sin();
        }
        let before: f64 = buffer.iter().map(|&v| (v as f64)*(v as f64) ).sum();


        let mut filter = HighPassFilter::new(sample_rate, cutoff_frequency);
        filter.process_buffer(&mut buffer);

        let after: f64 = buffer.iter().map(|&v| (v as f64)*(v as f64) ).sum();

        let energy_loss = (before - after).abs()/before;

        // Verify that the high pass filter has significantly reduced the low frequency content
        println!("Energy loss: {}%, before & after: {} / {} ", (energy_loss*100.0).round(), before, after);
        

        assert!(energy_loss > 0.8 , "High pass filter did not sufficiently reduce low frequency content");
    }


    #[test]
    fn test_highpass_filter_pass() {
        let sample_rate = 44100;
        let cutoff_frequency = 50.0;

        let mut buffer = vec![0.0; 10000];
        // Add some low frequency content to the buffer
        for (i, sample) in buffer.iter_mut().enumerate() {
            *sample += (i as f32 / sample_rate as f32 * 2.0 * std::f32::consts::PI * 500.0).sin();
        }
        let before: f64 = buffer.iter().map(|&v| (v as f64)*(v as f64) ).sum();


        let mut filter = HighPassFilter::new(sample_rate, cutoff_frequency);
        filter.process_buffer(&mut buffer);

        let after: f64 = buffer.iter().map(|&v| (v as f64)*(v as f64) ).sum();
        let energy_loss = (before - after).abs()/before;
        
        println!("Energy left: {}%, before & after: {} / {} ", (energy_loss*100.0).round(), before, after);

        assert!(energy_loss < 0.1, "High pass filter did not sufficiently reduce low frequency content");
    }
}