//! Global LFO with drift and nonlinear depth.

use crate::control::Parameters;

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum LfoWaveform {
    Triangle = 0,
    Saw = 1,
    Square = 2,
    Random = 3,
}

impl Default for LfoWaveform {
    fn default() -> Self {
        LfoWaveform::Triangle
    }
}

pub struct Lfo {
    phase: f32,
    rate: f32,
    drift: f32,
    drift_phase: f32,
    depth: f32,
    waveform: LfoWaveform,
    random_value: f32,
    random_target: f32,
    sample_hold_counter: f32,
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            rate: 1.0,
            drift: 0.0,
            drift_phase: 0.0,
            depth: 1.0,
            waveform: LfoWaveform::Triangle,
            random_value: 0.0,
            random_target: 0.0,
            sample_hold_counter: 0.0,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.phase = 0.0;
        self.drift = 0.0;
        self.drift_phase = 0.0;
        self.random_value = 0.0;
        self.random_target = 0.0;
        self.sample_hold_counter = 0.0;
    }

    pub fn process(&mut self, params: &Parameters) -> f32 {
        self.waveform = match params.lfo_wave {
            0 => LfoWaveform::Triangle,
            1 => LfoWaveform::Saw,
            2 => LfoWaveform::Square,
            _ => LfoWaveform::Random,
        };

        let rate_with_drift = self.rate * (1.0 + self.drift * 0.001);
        let nonlinear_depth = params.lfo_depth * (1.0 + (params.lfo_depth - 0.5) * 0.1);
        self.depth = nonlinear_depth;

        let phase_increment = rate_with_drift / 44100.0;
        self.phase += phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.update_random_target();
        }

        let output = self.generate_waveform();

        let drift_update = ((self.drift_phase * 7.13).sin() * 0.001).abs();
        self.drift_phase += 0.00001;
        if self.drift_phase >= 1.0 {
            self.drift_phase -= 1.0;
        }
        self.drift += drift_update;
        self.drift = self.drift.max(-10.0).min(10.0);

        output * self.depth
    }

    fn generate_waveform(&mut self) -> f32 {
        match self.waveform {
            LfoWaveform::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            LfoWaveform::Saw => 2.0 * self.phase - 1.0,
            LfoWaveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoWaveform::Random => {
                self.sample_hold_counter += 1.0;
                if self.sample_hold_counter >= 441.0 {
                    self.sample_hold_counter = 0.0;
                    self.random_value = self.random_target;
                }
                self.random_value
            }
        }
    }

    fn update_random_target(&mut self) {
        let x = (self.phase * 12.9898).sin() * 43758.5453;
        self.random_target = (x.fract() * 2.0) - 1.0;
    }

    pub fn update_params(&mut self, params: &Parameters) {
        self.rate = params.lfo_rate;
        self.depth = params.lfo_depth;
        self.waveform = match params.lfo_wave {
            0 => LfoWaveform::Triangle,
            1 => LfoWaveform::Saw,
            2 => LfoWaveform::Square,
            _ => LfoWaveform::Random,
        };
    }

    #[allow(dead_code)]
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    #[allow(dead_code)]
    pub fn set_waveform(&mut self, waveform: LfoWaveform) {
        self.waveform = waveform;
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}
