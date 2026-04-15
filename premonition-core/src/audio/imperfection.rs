//! Per-voice imperfection system (Slop).

const PI: f32 = 3.14159265359;

pub struct Imperfection {
    pub voice_index: usize,
    seed: u32,

    pub osc_drift: f32,
    pub osc_drift_phase: f32,
    pub osc_drift_rate: f32,

    pub filter_cutoff_offset: f32,

    pub envelope_timing_offset: f32,
    pub envelope_curve_variance: f32,

    pub mixer_gain_variance: f32,

    pub pan_position: f32,
    pub filter_env_offset: f32,
}

impl Imperfection {
    pub fn new(voice_index: usize) -> Self {
        let seed = (voice_index as u32)
            .wrapping_mul(2654435761)
            .wrapping_add(1);
        let pan = Self::seeded_random(seed) * 0.3 - 0.15;

        Self {
            voice_index,
            seed,
            osc_drift: 0.0,
            osc_drift_phase: Self::seeded_random(seed.wrapping_add(100)) * 0.1,
            osc_drift_rate: 0.00001 + Self::seeded_random(seed.wrapping_add(200)) * 0.0001,
            filter_cutoff_offset: (Self::seeded_random(seed.wrapping_add(300)) * 0.2) - 0.1,
            envelope_timing_offset: (Self::seeded_random(seed.wrapping_add(400)) * 0.1) - 0.05,
            envelope_curve_variance: (Self::seeded_random(seed.wrapping_add(500)) * 0.1) - 0.05,
            mixer_gain_variance: Self::seeded_random(seed.wrapping_add(600)) * 0.1,
            pan_position: pan,
            filter_env_offset: (Self::seeded_random(seed.wrapping_add(700)) * 0.1) - 0.05,
        }
    }

    pub fn init(&mut self) {
        self.osc_drift = 0.0;
        self.osc_drift_phase =
            Self::seeded_random(self.seed.wrapping_add(self.voice_index as u32)) * 0.1;
    }

    pub fn update(&mut self) {
        self.osc_drift_phase += self.osc_drift_rate;
        if self.osc_drift_phase >= 1.0 {
            self.osc_drift_phase -= 1.0;
        }

        let chaotic = (self.osc_drift_phase * PI * 2.0).sin()
            + (self.osc_drift_phase * 7.3).sin() * 0.5
            + (self.osc_drift_phase * 13.7).sin() * 0.25;

        self.osc_drift += chaotic * self.osc_drift_rate * 100.0;
        self.osc_drift = self.osc_drift.max(-50.0).min(50.0);
    }

    #[allow(dead_code)]
    pub fn apply_slop(&mut self, slop: f32) {
        let slop_factor = slop.max(0.0).min(1.0);

        self.osc_drift_rate = (0.00001 + Self::seeded_random(self.seed.wrapping_add(800)) * 0.0001)
            * (1.0 + slop_factor * 10.0);
        self.filter_cutoff_offset =
            ((Self::seeded_random(self.seed.wrapping_add(300)) * 0.2) - 0.1) * slop_factor;
        self.envelope_timing_offset =
            ((Self::seeded_random(self.seed.wrapping_add(400)) * 0.1) - 0.05) * slop_factor;
        self.envelope_curve_variance =
            ((Self::seeded_random(self.seed.wrapping_add(500)) * 0.1) - 0.05) * slop_factor;
        self.mixer_gain_variance =
            Self::seeded_random(self.seed.wrapping_add(600)) * 0.1 * slop_factor;
        self.filter_env_offset =
            ((Self::seeded_random(self.seed.wrapping_add(700)) * 0.1) - 0.05) * slop_factor;
    }

    #[allow(dead_code)]
    pub fn get_osc_drift(&self) -> f32 {
        self.osc_drift
    }

    #[allow(dead_code)]
    pub fn get_filter_cutoff_offset(&self) -> f32 {
        self.filter_cutoff_offset
    }

    #[allow(dead_code)]
    pub fn get_envelope_timing_offset(&self) -> f32 {
        self.envelope_timing_offset
    }

    #[allow(dead_code)]
    pub fn get_envelope_curve_variance(&self) -> f32 {
        self.envelope_curve_variance
    }

    #[allow(dead_code)]
    pub fn get_mixer_gain_variance(&self) -> f32 {
        self.mixer_gain_variance
    }

    #[allow(dead_code)]
    pub fn get_pan_position(&self) -> f32 {
        self.pan_position
    }

    #[allow(dead_code)]
    pub fn get_filter_env_offset(&self) -> f32 {
        self.filter_env_offset
    }

    fn seeded_random(seed: u32) -> f32 {
        let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((x >> 16) as u32 & 0x7FFF) as f32 / 32768.0
    }
}

impl Default for Imperfection {
    fn default() -> Self {
        Self::new(0)
    }
}
