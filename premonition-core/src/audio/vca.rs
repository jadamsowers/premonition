//! OTA-style voltage-controlled amplifier.

use num_traits::float::Float;

pub struct Vca {
    voice_index: usize,
    velocity_sensitivity: f32,
    gain: f32,
    distortion: f32,
    level_error: f32,
}

impl Vca {
    pub fn new(voice_index: usize) -> Self {
        let level_error = ((voice_index as f32 * 9.1).sin() * 0.02).abs();
        Self {
            voice_index,
            velocity_sensitivity: 1.0,
            gain: 1.0,
            distortion: 0.0,
            level_error,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.gain = 1.0;
    }

    pub fn process(&mut self, input: f32, envelope_level: f32) -> f32 {
        let velocity_mod = envelope_level * self.velocity_sensitivity;

        let linear_gain = velocity_mod * self.gain;

        let distortion_amount = linear_gain * self.distortion;
        let distorted_gain = linear_gain * (1.0 + distortion_amount * 0.1);

        let output = input * distorted_gain;

        let level_variance = 1.0 + self.level_error;
        let output_with_error = output * level_variance;

        output_with_error
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_velocity_sensitivity(&mut self, sensitivity: f32) {
        self.velocity_sensitivity = sensitivity;
    }

    pub fn set_distortion(&mut self, amount: f32) {
        self.distortion = amount;
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }
}

impl Default for Vca {
    fn default() -> Self {
        Self::new(0)
    }
}
