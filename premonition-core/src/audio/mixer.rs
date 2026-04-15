//! Three-input mixer with soft saturation and per-voice variance.

use crate::control::Parameters;
use num_traits::float::Float;

pub struct Mixer {
    voice_index: usize,
    osc1_gain: f32,
    osc2_gain: f32,
    noise_gain: f32,
    saturation: f32,
    dc_offset: f32,
}

impl Mixer {
    pub fn new(voice_index: usize) -> Self {
        let variance = ((voice_index as f32 * 5.31).sin() * 0.05).abs();
        Self {
            voice_index,
            osc1_gain: 1.0,
            osc2_gain: 1.0,
            noise_gain: 1.0,
            saturation: 0.9,
            dc_offset: ((voice_index as f32 * 3.7).sin() * 0.001),
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {}

    pub fn process(&mut self, osc1: f32, osc2: f32, noise: f32, params: &Parameters) -> (f32, f32) {
        let mix_ratio = params.osc_mix;

        let osc1_weighted = osc1 * (1.0 - mix_ratio);
        let osc2_weighted = osc2 * mix_ratio;
        let noise_weighted = noise * params.noise_level;

        let raw_mix = osc1_weighted * self.osc1_gain
            + osc2_weighted * self.osc2_gain
            + noise_weighted * self.noise_gain;

        let saturation_point = self.saturation;
        let saturated = self.soft_clip(raw_mix, saturation_point);

        let harmonic_distortion = self.add_harmonic_distortion(saturated);

        let output = harmonic_distortion + self.dc_offset;

        let harmonic_content = self.calculate_harmonic_content(raw_mix, saturated);

        (output, harmonic_content)
    }

    fn soft_clip(&self, input: f32, threshold: f32) -> f32 {
        if input.abs() < threshold {
            input
        } else {
            let sign = if input > 0.0 { 1.0 } else { -1.0 };
            let excess = input.abs() - threshold;
            let compressed = threshold + (excess / (1.0 + excess * 0.5));
            sign * compressed.min(2.0)
        }
    }

    fn add_harmonic_distortion(&self, input: f32) -> f32 {
        let even_harmonics = input + (input * input * input * 0.1);
        let odd_distortion = even_harmonics * (1.0 + (even_harmonics.abs() * 0.05));
        odd_distortion
    }

    fn calculate_harmonic_content(&self, clean: f32, distorted: f32) -> f32 {
        let thd = ((distorted - clean).powi(2) / (clean.powi(2) + 0.0001)).sqrt();
        thd.min(1.0)
    }

    pub fn set_osc1_gain(&mut self, gain: f32) {
        self.osc1_gain = gain;
    }

    pub fn set_osc2_gain(&mut self, gain: f32) {
        self.osc2_gain = gain;
    }

    pub fn set_noise_gain(&mut self, gain: f32) {
        self.noise_gain = gain;
    }

    pub fn set_saturation(&mut self, sat: f32) {
        self.saturation = sat;
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new(0)
    }
}
