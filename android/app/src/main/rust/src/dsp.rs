pub struct DspPipeline {
    #[allow(dead_code)]
    channels: usize,
    sample_rate: u32,
    eq_filters: Vec<BiquadFilter>,
}

impl DspPipeline {
    pub fn new(channels: usize, sample_rate: u32) -> Self {
        Self {
            channels,
            sample_rate,
            eq_filters: Vec::new(),
        }
    }

    pub fn set_eq_gains(&mut self, gains: &[f32]) {
        // Clear and recreate filters
        self.eq_filters.clear();
        
        // Define frequencies (matching the 16-band graphic EQ in JS)
        let frequencies = [
            25, 40, 63, 100, 160, 250, 400, 630, 
            1000, 1600, 2500, 4000, 6300, 10000, 16000, 20000
        ];

        for (i, &gain) in gains.iter().enumerate() {
            if i >= frequencies.len() { break; }
            let freq = frequencies[i] as f32;
            let filter = BiquadFilter::peaking(self.sample_rate as f32, freq, gain, 1.41);
            self.eq_filters.push(filter);
        }
    }

    pub fn process(&mut self, data: &mut [f32]) {
        for filter in &mut self.eq_filters {
            filter.process(data);
        }
    }
}

struct BiquadFilter {
    a1: f32, a2: f32,
    b0: f32, b1: f32, b2: f32,
    z1: f32, z2: f32,
}

impl BiquadFilter {
    // Standard peaking EQ implementation
    pub fn peaking(sample_rate: f32, frequency: f32, gain_db: f32, q: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
            z1: 0.0, z2: 0.0,
        }
    }

    pub fn process(&mut self, data: &mut [f32]) {
        for sample in data.iter_mut() {
            let out = self.b0 * (*sample) + self.z1;
            self.z1 = self.b1 * (*sample) - self.a1 * out + self.z2;
            self.z2 = self.b2 * (*sample) - self.a2 * out;
            *sample = out;
        }
    }
}
