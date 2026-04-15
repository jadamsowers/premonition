//! Voltage-controlled oscillator (VCO) implementation.

use crate::control::Parameters;

#[allow(dead_code)]
const PI: f32 = 3.14159265359;

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Waveform {
    Saw = 0,
    Pulse = 1,
    Triangle = 2,
}

impl Default for Waveform {
    fn default() -> Self {
        Waveform::Saw
    }
}

pub struct Oscillator {
    voice_index: usize,
    phase: f32,
    sync_phase: f32,
    base_frequency: f32,
    current_frequency: f32,
    pitch_mod: f32,
    pulse_width: f32,
    drift: f32,
    drift_phase: f32,
    hard_sync_enabled: bool,
    last_value: f32,
}

impl Oscillator {
    pub fn new(voice_index: usize) -> Self {
        Self {
            voice_index,
            phase: 0.0,
            sync_phase: 0.0,
            base_frequency: 440.0,
            current_frequency: 440.0,
            pitch_mod: 1.0,
            pulse_width: 0.5,
            drift: 0.0,
            drift_phase: (voice_index as f32 * 0.1).fract(),
            hard_sync_enabled: false,
            last_value: 0.0,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.phase = 0.0;
        self.drift = 0.0;
        self.drift_phase = (self.voice_index as f32 * 0.1).fract();
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.base_frequency = frequency;
    }

    pub fn set_pitch_mod(&mut self, mod_value: f32) {
        self.pitch_mod = mod_value;
    }

    pub fn set_pulse_width(&mut self, pw: f32) {
        let pw_sqrt = pw.sqrt();
        self.pulse_width = 0.1 + (pw_sqrt * 0.8);
    }

    pub fn set_hard_sync(&mut self, enabled: bool) {
        self.hard_sync_enabled = enabled;
    }

    pub fn process(&mut self, waveform: Waveform) -> f32 {
        let drift_factor = 1.0 + (self.drift * 0.001);
        self.current_frequency = self.base_frequency * self.pitch_mod * drift_factor;

        let phase_increment = self.current_frequency / 44100.0;
        self.phase += phase_increment;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
            if self.hard_sync_enabled {
                self.sync_phase = 0.0;
            }
        }

        let output = match waveform {
            Waveform::Saw => self.generate_saw(),
            Waveform::Pulse => self.generate_pulse(),
            Waveform::Triangle => self.generate_triangle(),
        };

        self.last_value = output;
        output
    }

    fn generate_saw(&self) -> f32 {
        2.0 * self.phase - 1.0
    }

    fn generate_pulse(&self) -> f32 {
        if self.phase < self.pulse_width {
            1.0
        } else {
            -1.0
        }
    }

    fn generate_triangle(&self) -> f32 {
        let triangle = if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - (4.0 * self.phase)
        };
        let harmonics = triangle * 0.15 * (self.phase * 10.0).sin();
        triangle * 0.85 + harmonics
    }

    pub fn sync_to(&mut self, source: &Oscillator) {
        if source.phase < source.last_value && source.last_value > 0.0 {
            self.phase = 0.0;
        }
    }

    pub fn update_drift(&mut self, slop: f32) {
        self.drift_phase += 0.0001;
        if self.drift_phase >= 1.0 {
            self.drift_phase -= 1.0;
        }

        let drift_rate = 0.00001 + (slop * 0.0001);
        self.drift += (Self::pseudo_random(self.drift_phase) - 0.5) * drift_rate * 1000.0;
        self.drift = self.drift.max(-50.0).min(50.0);
    }

    fn pseudo_random(phase: f32) -> f32 {
        let x = (phase * 12.9898).sin() * 43758.5453;
        x.fract()
    }

    #[allow(dead_code)]
    pub fn update_params(&mut self, _params: &Parameters) {}

    pub fn get_phase(&self) -> f32 {
        self.phase
    }
}

impl Default for Oscillator {
    fn default() -> Self {
        Self::new(0)
    }
}
