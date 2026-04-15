//! Parameter definitions and storage.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ParameterId {
    Osc1Wave = 0,
    Osc1Freq = 1,
    Osc1Fine = 2,
    Osc1PulseWidth = 3,
    Osc1PitchEnvDepth = 4,
    Osc2Wave = 5,
    Osc2Coarse = 6,
    Osc2Fine = 7,
    Osc2PulseWidth = 8,
    NoiseLevel = 9,
    OscMix = 10,
    FilterCutoff = 11,
    FilterResonance = 12,
    FilterEnvAmount = 13,
    FilterKeyTracking = 14,
    AmpAttack = 15,
    AmpDecay = 16,
    AmpSustain = 17,
    AmpRelease = 18,
    FilterAttack = 19,
    FilterDecay = 20,
    FilterSustain = 21,
    FilterRelease = 22,
    LfoRate = 23,
    LfoWave = 24,
    LfoDepth = 25,
    LfoToOsc = 26,
    LfoToFilter = 27,
    LfoToPw = 28,
    PolyModOsc2ToOsc1Freq = 29,
    PolyModOsc2ToOsc1Pw = 30,
    PolyModFilterEnvToFilter = 31,
    Slop = 32,
    UnisonVoices = 33,
    UnisonDetune = 34,
    MasterVolume = 35,
}

impl ParameterId {
    pub fn from_raw(id: u32) -> Self {
        match id {
            0 => ParameterId::Osc1Wave,
            1 => ParameterId::Osc1Freq,
            2 => ParameterId::Osc1Fine,
            3 => ParameterId::Osc1PulseWidth,
            4 => ParameterId::Osc1PitchEnvDepth,
            5 => ParameterId::Osc2Wave,
            6 => ParameterId::Osc2Coarse,
            7 => ParameterId::Osc2Fine,
            8 => ParameterId::Osc2PulseWidth,
            9 => ParameterId::NoiseLevel,
            10 => ParameterId::OscMix,
            11 => ParameterId::FilterCutoff,
            12 => ParameterId::FilterResonance,
            13 => ParameterId::FilterEnvAmount,
            14 => ParameterId::FilterKeyTracking,
            15 => ParameterId::AmpAttack,
            16 => ParameterId::AmpDecay,
            17 => ParameterId::AmpSustain,
            18 => ParameterId::AmpRelease,
            19 => ParameterId::FilterAttack,
            20 => ParameterId::FilterDecay,
            21 => ParameterId::FilterSustain,
            22 => ParameterId::FilterRelease,
            23 => ParameterId::LfoRate,
            24 => ParameterId::LfoWave,
            25 => ParameterId::LfoDepth,
            26 => ParameterId::LfoToOsc,
            27 => ParameterId::LfoToFilter,
            28 => ParameterId::LfoToPw,
            29 => ParameterId::PolyModOsc2ToOsc1Freq,
            30 => ParameterId::PolyModOsc2ToOsc1Pw,
            31 => ParameterId::PolyModFilterEnvToFilter,
            32 => ParameterId::Slop,
            33 => ParameterId::UnisonVoices,
            34 => ParameterId::UnisonDetune,
            35 => ParameterId::MasterVolume,
            _ => ParameterId::MasterVolume,
        }
    }

    pub fn raw(&self) -> u32 {
        *self as u32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub osc1_wave: u8,
    pub osc1_freq: f32,
    pub osc1_fine: f32,
    pub osc1_pulse_width: f32,
    pub osc1_pitch_env_depth: f32,

    pub osc2_wave: u8,
    pub osc2_coarse: f32,
    pub osc2_fine: f32,
    pub osc2_pulse_width: f32,

    pub noise_level: f32,
    pub osc_mix: f32,

    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub filter_env_amount: f32,
    pub filter_keyboard_tracking: f32,

    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,

    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,

    pub lfo_rate: f32,
    pub lfo_wave: u8,
    pub lfo_depth: f32,
    pub lfo_to_osc: f32,
    pub lfo_to_filter: f32,
    pub lfo_to_pw: f32,

    pub poly_mod_osc2_to_osc1_freq: f32,
    pub poly_mod_osc2_to_osc1_pw: f32,
    pub poly_mod_filter_env_to_filter: f32,

    pub slop: f32,
    pub unison_voices: u8,
    pub unison_detune: f32,

    pub master_volume: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            osc1_wave: 0,
            osc1_freq: 0.0,
            osc1_fine: 0.0,
            osc1_pulse_width: 0.5,
            osc1_pitch_env_depth: 0.0,

            osc2_wave: 0,
            osc2_coarse: 0.0,
            osc2_fine: 0.0,
            osc2_pulse_width: 0.5,

            noise_level: 0.0,
            osc_mix: 0.5,

            filter_cutoff: 5000.0,
            filter_resonance: 0.0,
            filter_env_amount: 0.0,
            filter_keyboard_tracking: 0.0,

            amp_attack: 0.001,
            amp_decay: 0.2,
            amp_sustain: 0.5,
            amp_release: 0.3,

            filter_attack: 0.001,
            filter_decay: 0.2,
            filter_sustain: 0.5,
            filter_release: 0.3,

            lfo_rate: 1.0,
            lfo_wave: 0,
            lfo_depth: 0.5,
            lfo_to_osc: 0.0,
            lfo_to_filter: 0.0,
            lfo_to_pw: 0.0,

            poly_mod_osc2_to_osc1_freq: 0.0,
            poly_mod_osc2_to_osc1_pw: 0.0,
            poly_mod_filter_env_to_filter: 0.0,

            slop: 0.1,
            unison_voices: 1,
            unison_detune: 0.0,

            master_volume: 1.0,
        }
    }
}

impl Parameters {
    pub fn set(&mut self, param: ParameterId, value: f32) {
        match param {
            ParameterId::Osc1Wave => self.osc1_wave = (value * 1.0).round() as u8,
            ParameterId::Osc1Freq => self.osc1_freq = (value - 0.5) * 48.0,
            ParameterId::Osc1Fine => self.osc1_fine = (value - 0.5) * 2.0,
            ParameterId::Osc1PulseWidth => self.osc1_pulse_width = value,
            ParameterId::Osc1PitchEnvDepth => self.osc1_pitch_env_depth = (value - 0.5) * 0.5,
            ParameterId::Osc2Wave => self.osc2_wave = (value * 2.0).round() as u8,
            ParameterId::Osc2Coarse => self.osc2_coarse = (value - 0.5) * 48.0,
            ParameterId::Osc2Fine => self.osc2_fine = (value - 0.5) * 2.0,
            ParameterId::Osc2PulseWidth => self.osc2_pulse_width = value,
            ParameterId::NoiseLevel => self.noise_level = value,
            ParameterId::OscMix => self.osc_mix = value,
            ParameterId::FilterCutoff => self.filter_cutoff = 20.0 + value * 19980.0,
            ParameterId::FilterResonance => self.filter_resonance = value,
            ParameterId::FilterEnvAmount => self.filter_env_amount = (value - 0.5) * 2.0,
            ParameterId::FilterKeyTracking => self.filter_keyboard_tracking = value,
            ParameterId::AmpAttack => self.amp_attack = value.powf(3.0) * 10.0,
            ParameterId::AmpDecay => self.amp_decay = value.powf(3.0) * 10.0,
            ParameterId::AmpSustain => self.amp_sustain = value,
            ParameterId::AmpRelease => self.amp_release = value.powf(3.0) * 10.0,
            ParameterId::FilterAttack => self.filter_attack = value.powf(3.0) * 10.0,
            ParameterId::FilterDecay => self.filter_decay = value.powf(3.0) * 10.0,
            ParameterId::FilterSustain => self.filter_sustain = value,
            ParameterId::FilterRelease => self.filter_release = value.powf(3.0) * 10.0,
            ParameterId::LfoRate => self.lfo_rate = 0.01 + value.powf(2.0) * 100.0,
            ParameterId::LfoWave => self.lfo_wave = (value * 3.0).round() as u8,
            ParameterId::LfoDepth => self.lfo_depth = value,
            ParameterId::LfoToOsc => self.lfo_to_osc = (value - 0.5) * 0.1,
            ParameterId::LfoToFilter => self.lfo_to_filter = value * 5000.0,
            ParameterId::LfoToPw => self.lfo_to_pw = (value - 0.5) * 0.5,
            ParameterId::PolyModOsc2ToOsc1Freq => {
                self.poly_mod_osc2_to_osc1_freq = (value - 0.5) * 2.0
            }
            ParameterId::PolyModOsc2ToOsc1Pw => self.poly_mod_osc2_to_osc1_pw = (value - 0.5) * 2.0,
            ParameterId::PolyModFilterEnvToFilter => {
                self.poly_mod_filter_env_to_filter = value - 0.5
            }
            ParameterId::Slop => self.slop = value,
            ParameterId::UnisonVoices => self.unison_voices = (value * 7.0) as u8 + 1,
            ParameterId::UnisonDetune => self.unison_detune = value,
            ParameterId::MasterVolume => self.master_volume = value,
        }
    }

    pub fn get(&self, param: ParameterId) -> f32 {
        match param {
            ParameterId::Osc1Wave => self.osc1_wave as f32 / 1.0,
            ParameterId::Osc1Freq => (self.osc1_freq / 48.0) + 0.5,
            ParameterId::Osc1Fine => (self.osc1_fine / 2.0) + 0.5,
            ParameterId::Osc1PulseWidth => self.osc1_pulse_width,
            ParameterId::Osc1PitchEnvDepth => (self.osc1_pitch_env_depth / 0.5) + 0.5,
            ParameterId::Osc2Wave => self.osc2_wave as f32 / 2.0,
            ParameterId::Osc2Coarse => (self.osc2_coarse / 48.0) + 0.5,
            ParameterId::Osc2Fine => (self.osc2_fine / 2.0) + 0.5,
            ParameterId::Osc2PulseWidth => self.osc2_pulse_width,
            ParameterId::NoiseLevel => self.noise_level,
            ParameterId::OscMix => self.osc_mix,
            ParameterId::FilterCutoff => (self.filter_cutoff - 20.0) / 19980.0,
            ParameterId::FilterResonance => self.filter_resonance,
            ParameterId::FilterEnvAmount => (self.filter_env_amount / 2.0) + 0.5,
            ParameterId::FilterKeyTracking => self.filter_keyboard_tracking,
            ParameterId::AmpAttack => (self.amp_attack / 10.0).powf(1.0 / 3.0),
            ParameterId::AmpDecay => (self.amp_decay / 10.0).powf(1.0 / 3.0),
            ParameterId::AmpSustain => self.amp_sustain,
            ParameterId::AmpRelease => (self.amp_release / 10.0).powf(1.0 / 3.0),
            ParameterId::FilterAttack => (self.filter_attack / 10.0).powf(1.0 / 3.0),
            ParameterId::FilterDecay => (self.filter_decay / 10.0).powf(1.0 / 3.0),
            ParameterId::FilterSustain => self.filter_sustain,
            ParameterId::FilterRelease => (self.filter_release / 10.0).powf(1.0 / 3.0),
            ParameterId::LfoRate => ((self.lfo_rate - 0.01) / 100.0).sqrt(),
            ParameterId::LfoWave => self.lfo_wave as f32 / 3.0,
            ParameterId::LfoDepth => self.lfo_depth,
            ParameterId::LfoToOsc => (self.lfo_to_osc / 0.1) + 0.5,
            ParameterId::LfoToFilter => self.lfo_to_filter / 5000.0,
            ParameterId::LfoToPw => (self.lfo_to_pw / 0.5) + 0.5,
            ParameterId::PolyModOsc2ToOsc1Freq => (self.poly_mod_osc2_to_osc1_freq / 2.0) + 0.5,
            ParameterId::PolyModOsc2ToOsc1Pw => (self.poly_mod_osc2_to_osc1_pw / 2.0) + 0.5,
            ParameterId::PolyModFilterEnvToFilter => self.poly_mod_filter_env_to_filter + 0.5,
            ParameterId::Slop => self.slop,
            ParameterId::UnisonVoices => (self.unison_voices as f32 - 1.0) / 7.0,
            ParameterId::UnisonDetune => self.unison_detune,
            ParameterId::MasterVolume => self.master_volume,
        }
    }

    pub fn to_json(&self) -> Result<alloc::string::String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_serialization() {
        let mut params = Parameters::default();
        params.master_volume = 0.42;
        params.osc1_wave = 1;
        params.filter_cutoff = 12000.0;

        let json = params.to_json().expect("Failed to serialize");
        
        let loaded = Parameters::from_json(&json).expect("Failed to deserialize");
        assert_eq!(loaded.master_volume, 0.42);
        assert_eq!(loaded.osc1_wave, 1);
        assert_eq!(loaded.filter_cutoff, 12000.0);
    }
    
    #[test]
    fn test_parameter_normalization() {
        let mut params = Parameters::default();
        
        // Master Volume bounds 0.0 to 1.0
        params.set(ParameterId::MasterVolume, 0.5);
        assert_eq!(params.master_volume, 0.5);
        
        // Filter Cutoff bounds 20.0 to 20000.0
        params.set(ParameterId::FilterCutoff, 0.5);
        // (19980 * 0.5) + 20 = 10010.0
        assert_eq!(params.filter_cutoff, 10010.0);
        
        // Ensure get() reconstructs the 0..1 normalized mapping properly
        assert_eq!(params.get(ParameterId::FilterCutoff), 0.5);
    }
}
