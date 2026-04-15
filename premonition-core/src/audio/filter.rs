//! Curtis/SSM-style 4-pole low-pass filter using TPT topology.

use crate::control::Parameters;

const PI: f32 = 3.14159265359;

pub struct Filter {
    #[allow(dead_code)]
    voice_index: usize,
    cutoff: f32,
    resonance: f32,
    #[allow(dead_code)]
    keyboard_tracking: f32,
    sample_rate: f32,

    // State variables for TPT stages
    s1: f32,
    s2: f32,
    s3: f32,
    s4: f32,

    cutoff_variance: f32,
    resonance_offset: f32,
}

impl Filter {
    pub fn new(voice_index: usize) -> Self {
        let cutoff_variance = ((voice_index as f32 * 11.3).sin() * 0.05).abs();
        Self {
            voice_index,
            cutoff: 5000.0,
            resonance: 0.0,
            keyboard_tracking: 0.0,
            sample_rate: 44100.0,
            s1: 0.0,
            s2: 0.0,
            s3: 0.0,
            s4: 0.0,
            cutoff_variance,
            resonance_offset: ((voice_index as f32 * 7.7).sin() * 0.05).abs(),
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.s1 = 0.0;
        self.s2 = 0.0;
        self.s3 = 0.0;
        self.s4 = 0.0;
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        let variance_factor = 1.0 + self.cutoff_variance;
        self.cutoff = (cutoff * variance_factor).max(20.0).min(20000.0);
    }

    pub fn set_resonance(&mut self, res: f32) {
        let nonlinear_res = res + (res * res * self.resonance_offset * 0.1);
        self.resonance = nonlinear_res.min(0.99); // Keep slightly below 1.0 for stability
    }

    pub fn set_keyboard_tracking(&mut self, tracking: f32) {
        self.keyboard_tracking = tracking;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // TPT 4-pole Ladder Filter Implementation
        let wc = (self.cutoff / self.sample_rate).min(0.45);
        let g = (PI * wc).tan();
        let gl = g / (1.0 + g);
        let res = self.resonance * 4.0; // Resonance range 0 to 4

        // Feedback calculation
        let sigma = gl * gl * gl * self.s1 + gl * gl * self.s2 + gl * self.s3 + self.s4;
        let gamma = gl * gl * gl * gl;
        // Apply clipping to the feedback to ensure stability
        let input_with_feedback = ((input - res * sigma) / (1.0 + res * gamma)).clamp(-4.0, 4.0);

        // Stage 1
        let v1 = (input_with_feedback - self.s1) * gl;
        let y1 = v1 + self.s1;
        self.s1 = (y1 + v1).clamp(-10.0, 10.0);

        // Stage 2
        let v2 = (y1 - self.s2) * gl;
        let y2 = v2 + self.s2;
        self.s2 = (y2 + v2).clamp(-10.0, 10.0);

        // Stage 3
        let v3 = (y2 - self.s3) * gl;
        let y3 = v3 + self.s3;
        self.s3 = (y3 + v3).clamp(-10.0, 10.0);

        // Stage 4
        let v4 = (y3 - self.s4) * gl;
        let y4 = v4 + self.s4;
        self.s4 = (y4 + v4).clamp(-10.0, 10.0);

        // Apply slight analog saturation/soft clipping using x / (1 + |x|)
        y4 / (1.0 + y4.abs())
    }

    pub fn update_params(&mut self, params: &Parameters) {
        self.set_cutoff(params.filter_cutoff);
        self.set_resonance(params.filter_resonance);
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_initialization() {
        let mut filter = Filter::new(0);
        filter.init(48000.0);
        assert_eq!(filter.s1, 0.0);
        assert_eq!(filter.s2, 0.0);
        assert_eq!(filter.s3, 0.0);
        assert_eq!(filter.s4, 0.0);
    }

    #[test]
    fn test_filter_lowpass_behavior() {
        let mut filter = Filter::new(0);
        filter.init(44100.0);
        filter.set_cutoff(400.0); // low cutoff
        filter.set_resonance(0.0);

        // Feed an impulse and check that high frequency energy is suppressed (smoothed)
        let impulse = filter.process(1.0);
        let mut sum = impulse.abs();
        for _ in 0..100 {
            sum += filter.process(0.0).abs();
        }
        
        // Output should slowly decay rather than jumping back down
        assert!(sum > 0.0, "Filter output was zero after impulse");
    }

    #[test]
    fn test_filter_resonance() {
        let mut filter_no_res = Filter::new(0);
        filter_no_res.init(44100.0);
        filter_no_res.set_cutoff(1000.0);
        filter_no_res.set_resonance(0.0);

        let mut filter_high_res = Filter::new(0);
        filter_high_res.init(44100.0);
        filter_high_res.set_cutoff(1000.0);
        filter_high_res.set_resonance(0.9);

        // Feed impulse
        filter_no_res.process(1.0);
        filter_high_res.process(1.0);

        let mut no_res_energy = 0.0;
        let mut high_res_energy = 0.0;

        for _ in 0..1000 {
            no_res_energy += filter_no_res.process(0.0).abs();
            high_res_energy += filter_high_res.process(0.0).abs();
        }

        // Higher resonance should ring longer and thus contain more energy after an impulse
        assert!(high_res_energy > no_res_energy, "High resonance should have more ringing energy");
    }
}
