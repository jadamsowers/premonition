//! ADSR envelope generator with analog-style timing and imperfections.

use crate::control::Parameters;

#[derive(Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelope {
    pub value: f32,
    voice_index: usize,
    stage: EnvelopeStage,
    rate: f32,

    attack_time: f32,
    decay_time: f32,
    sustain_level: f32,
    release_time: f32,

    stage_value: f32,

    min_attack_ms: f32,
    timing_jitter: f32,
    curve_variance: f32,
    sustain_variance: f32,

    retrigger_counter: f32,
}

impl Envelope {
    pub fn new(voice_index: usize) -> Self {
        Self {
            voice_index,
            stage: EnvelopeStage::Idle,
            value: 0.0,
            rate: 0.0,
            attack_time: 0.001,
            decay_time: 0.2,
            sustain_level: 0.5,
            release_time: 0.3,
            stage_value: 0.0,
            min_attack_ms: 2.0 + (voice_index as f32 % 3.0) * 0.7,
            timing_jitter: ((voice_index as f32 * 6.7).sin() * 0.1).abs(),
            curve_variance: ((voice_index as f32 * 4.3).sin() * 0.05).abs(),
            sustain_variance: ((voice_index as f32 * 8.1).sin() * 0.02).abs(),
            retrigger_counter: 0.0,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.value = 0.0;
        self.stage = EnvelopeStage::Idle;
    }

    pub fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.stage_value = 0.0;
        self.retrigger_counter = self.timing_jitter * 10.0;
    }

    pub fn release(&mut self) {
        self.stage = EnvelopeStage::Release;
        self.stage_value = self.value;
    }

    pub fn process(&mut self, _params: &Parameters) {
        if self.retrigger_counter > 0.0 {
            self.retrigger_counter -= 1.0;
            return;
        }

        let curve = 1.0 + self.curve_variance;

        match self.stage {
            EnvelopeStage::Idle => {
                self.value = 0.0;
            }
            EnvelopeStage::Attack => {
                let min_attack = self.min_attack_ms / 1000.0;
                let effective_attack = (self.attack_time.max(min_attack)).max(0.0001);
                let attack_rate = 1.0 / (effective_attack * 44100.0);

                self.value += attack_rate.powf(curve);
                self.value = self.value.min(1.0);

                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = EnvelopeStage::Decay;
                    self.stage_value = 0.0;
                }
            }
            EnvelopeStage::Decay => {
                let effective_decay = self.decay_time.max(0.001);
                let decay_rate = 1.0 / (effective_decay * 44100.0);
                let target_sustain = self.sustain_level + self.sustain_variance;

                self.value -= decay_rate.powf(curve);
                self.value = self.value.max(target_sustain);

                if self.value <= target_sustain {
                    self.value = target_sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.value = self.sustain_level + self.sustain_variance;
            }
            EnvelopeStage::Release => {
                let effective_release = self.release_time.max(0.001);
                let release_rate = 1.0 / (effective_release * 44100.0);

                self.value -= release_rate.powf(curve);
                self.value = self.value.max(0.0);

                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
    }

    pub fn is_idle(&self) -> bool {
        self.stage == EnvelopeStage::Idle && self.value < 0.001
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        !self.is_idle()
    }

    #[allow(dead_code)]
    pub fn get_stage(&self) -> EnvelopeStage {
        self.stage
    }

    pub fn update_params(&mut self, params: &Parameters) {
        self.attack_time = params.amp_attack;
        self.decay_time = params.amp_decay;
        self.sustain_level = params.amp_sustain;
        self.release_time = params.amp_release;
    }

    #[allow(dead_code)]
    pub fn update_filter_params(&mut self, params: &Parameters) {
        self.attack_time = params.filter_attack;
        self.decay_time = params.filter_decay;
        self.sustain_level = params.filter_sustain;
        self.release_time = params.filter_release;
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new(0)
    }
}
