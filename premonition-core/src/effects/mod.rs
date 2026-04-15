//! Effects processing modules.

mod chorus;
mod delay;
mod reverb;

pub use chorus::Chorus;
pub use delay::Delay;
pub use reverb::Reverb;

pub struct EffectsChain {
    chorus: Chorus,
    delay: Delay,
    reverb: Reverb,
    enabled: bool,
}

impl EffectsChain {
    pub fn new() -> Self {
        Self {
            chorus: Chorus::new(),
            delay: Delay::new(),
            reverb: Reverb::new(),
            enabled: false,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.chorus.init(sample_rate);
        self.delay.init(sample_rate);
        self.reverb.init(sample_rate);
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        let with_chorus = self.chorus.process(input);
        let with_delay = self.delay.process_mono(with_chorus);
        self.reverb.process_sample(with_delay)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for EffectsChain {
    fn default() -> Self {
        Self::new()
    }
}
