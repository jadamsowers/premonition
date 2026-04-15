//! Curtis/SSM-style 4-pole low-pass filter.

use crate::control::Parameters;

const PI: f32 = 3.14159265359;

pub struct Filter {
    voice_index: usize,
    cutoff: f32,
    resonance: f32,
    keyboard_tracking: f32,

    z1: f32,
    z2: f32,
    z3: f32,
    z4: f32,

    cutoff_variance: f32,
    resonance_offset: f32,
    self_osc_amp: f32,
    self_osc_phase: f32,

    input_history: f32,
    output_history: f32,
}

impl Filter {
    pub fn new(voice_index: usize) -> Self {
        let cutoff_variance = ((voice_index as f32 * 11.3).sin() * 0.1).abs();
        Self {
            voice_index,
            cutoff: 5000.0,
            resonance: 0.0,
            keyboard_tracking: 0.0,
            z1: 0.0,
            z2: 0.0,
            z3: 0.0,
            z4: 0.0,
            cutoff_variance,
            resonance_offset: ((voice_index as f32 * 7.7).sin() * 0.05).abs(),
            self_osc_amp: 0.0,
            self_osc_phase: 0.0,
            input_history: 0.0,
            output_history: 0.0,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.z1 = 0.0;
        self.z2 = 0.0;
        self.z3 = 0.0;
        self.z4 = 0.0;
        self.self_osc_phase = 0.0;
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        let variance_factor = 1.0 + self.cutoff_variance;
        self.cutoff = (cutoff * variance_factor).max(20.0).min(20000.0);
    }

    pub fn set_resonance(&mut self, res: f32) {
        let nonlinear_res = res + (res * res * self.resonance_offset * 0.1);
        self.resonance = nonlinear_res.min(1.0);
    }

    pub fn set_keyboard_tracking(&mut self, tracking: f32) {
        self.keyboard_tracking = tracking;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let note_pitch = 60.0;
        let tracking_mod = if self.keyboard_tracking > 0.0 {
            1.0 + (note_pitch - 60.0) / 120.0 * self.keyboard_tracking
        } else {
            1.0
        };

        let effective_cutoff = self.cutoff * tracking_mod;
        let wc = effective_cutoff / 44100.0;

        let g = (PI * wc).tan();
        let k = (2.0 - 4.0 * self.resonance).max(0.0).min(2.0);

        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let _a3 = g * a2;

        let input_with_feedback = input - (self.z4 * k);

        let ic1eq = self.z1;
        let ic2eq = self.z2;
        let ic3eq = self.z3;

        self.z1 = input_with_feedback * a2 + ic1eq * a1 - ic1eq;
        self.z1 = self.z1.max(-10.0).min(10.0);

        self.z2 = self.z1 * a2 + ic2eq * a1 - ic2eq;
        self.z2 = self.z2.max(-10.0).min(10.0);

        self.z3 = self.z2 * a2 + ic3eq * a1 - ic3eq;
        self.z3 = self.z3.max(-10.0).min(10.0);

        self.z4 = self.z3 * a2 + self.z4 * a1 - self.z4;
        self.z4 = self.z4.max(-10.0).min(10.0);

        let output = self.z4;

        self.input_history = self.input_history * 0.99 + input * 0.01;
        self.output_history = self.output_history * 0.99 + output * 0.01;

        let level_interaction = 1.0 + (input.abs() - 0.5) * 0.1;
        let resonance_boost = self.resonance * level_interaction;

        let envelope_depth_effect = 1.0 + (resonance_boost - 0.5) * 0.05;

        output * envelope_depth_effect
    }

    #[allow(dead_code)]
    pub fn process_self_oscillation(&mut self) -> f32 {
        if self.resonance > 0.9 {
            let osc_amp_target = (self.resonance - 0.9) * 10.0;
            self.self_osc_amp = self.self_osc_amp * 0.999 + osc_amp_target * 0.001;

            let instability = 1.0 + ((self.voice_index as f32 * 13.3).sin() * 0.001);
            self.self_osc_phase += 0.01 * instability;
            if self.self_osc_phase >= 1.0 {
                self.self_osc_phase -= 1.0;
            }

            let sine = (self.self_osc_phase * PI * 2.0).sin();
            sine * self.self_osc_amp * 0.5
        } else {
            self.self_osc_amp *= 0.95;
            0.0
        }
    }

    pub fn update_params(&mut self, params: &Parameters) {
        self.set_cutoff(params.filter_cutoff);
        self.set_resonance(params.filter_resonance);
    }

    #[allow(dead_code)]
    pub fn get_cutoff(&self) -> f32 {
        self.cutoff
    }

    #[allow(dead_code)]
    pub fn get_resonance(&self) -> f32 {
        self.resonance
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new(0)
    }
}
