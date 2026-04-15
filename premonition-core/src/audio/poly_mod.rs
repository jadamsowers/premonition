//! Poly-Mod section for cross-modulation.

use num_traits::float::Float;

pub struct PolyMod {
    voice_index: usize,

    osc2_depth: f32,
    filter_env_depth: f32,

    asymmetry: f32,
    scaling_error: f32,

    last_osc2_sample: f32,
}

impl PolyMod {
    pub fn new(voice_index: usize) -> Self {
        Self {
            voice_index,
            osc2_depth: 0.0,
            filter_env_depth: 0.0,
            asymmetry: 0.0,
            scaling_error: ((voice_index as f32 * 5.7).sin() * 0.1).abs(),
            last_osc2_sample: 0.0,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.last_osc2_sample = 0.0;
    }

    pub fn process(
        &mut self,
        osc2_sample: f32,
        filter_env_value: f32,
        osc1_freq_mod: f32,
        osc1_pw_mod: f32,
        filter_mod: f32,
    ) -> f32 {
        let bandwidth_limited_osc2 = (self.last_osc2_sample + osc2_sample) * 0.5;
        self.last_osc2_sample = osc2_sample;

        let osc2_contribution =
            bandwidth_limited_osc2 * self.osc2_depth * self.get_asymmetry_factor();

        let env_contribution =
            filter_env_value * self.filter_env_depth * (1.0 + self.scaling_error);

        let total_mod = osc2_contribution + env_contribution;

        total_mod
    }

    pub fn get_osc1_freq_mod(&self, osc2_sample: f32, params: &[f32; 3]) -> f32 {
        let depth = params[0];
        let bandwidth_limited = (self.last_osc2_sample + osc2_sample) * 0.5;
        bandwidth_limited * depth * self.get_asymmetry_factor()
    }

    pub fn get_osc1_pw_mod(&self, filter_env: f32, depth: f32) -> f32 {
        filter_env * depth * (1.0 + self.scaling_error)
    }

    pub fn get_filter_mod(&self) -> f32 {
        0.0
    }

    fn get_asymmetry_factor(&self) -> f32 {
        1.0 + self.asymmetry * 0.5
    }

    pub fn set_osc2_depth(&mut self, depth: f32) {
        self.osc2_depth = depth;
    }

    pub fn set_filter_env_depth(&mut self, depth: f32) {
        self.filter_env_depth = depth;
    }

    pub fn set_asymmetry(&mut self, asymmetry: f32) {
        self.asymmetry = asymmetry;
    }

    pub fn update_from_params(&mut self, osc1_freq: f32, osc1_pw: f32, filter: f32) {
        self.osc2_depth = osc1_freq;
        self.filter_env_depth = osc1_pw;
    }
}

impl Default for PolyMod {
    fn default() -> Self {
        Self::new(0)
    }
}
