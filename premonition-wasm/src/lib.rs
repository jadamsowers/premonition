//! WebAssembly bindings for Premonition synthesizer.

use premonition_core::prelude::*;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

#[wasm_bindgen]
pub struct WasmEngine {
    engine: Engine,
    sample_rate: f32,
}

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let engine = Engine::new();
        Self {
            engine,
            sample_rate: 44100.0,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.engine.init(sample_rate);
    }

    pub fn process(&mut self, output_l: &mut [f32], output_r: &mut [f32]) {
        let num_samples = output_l.len().min(output_r.len());
        self.engine
            .process(output_l, output_r, num_samples, self.sample_rate);
    }

    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        self.engine.handle_midi(MidiMessage::NoteOn {
            channel,
            note,
            velocity,
        });
    }

    pub fn note_off(&mut self, channel: u8, note: u8) {
        self.engine
            .handle_midi(MidiMessage::NoteOff { channel, note });
    }

    pub fn set_param(&mut self, param_id: u32, value: f32) {
        self.engine
            .set_parameter(ParameterId::from_raw(param_id), value);
    }

    pub fn get_param(&self, param_id: u32) -> f32 {
        self.engine.get_parameter(ParameterId::from_raw(param_id))
    }

    pub fn set_params_json(&mut self, json: &str) -> Result<(), JsValue> {
        match Parameters::from_json(json) {
            Ok(params) => {
                let engine_params = self.engine.get_params_mut();
                *engine_params = params;
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to parse params: {}", e))),
        }
    }

    pub fn get_params_json(&self) -> String {
        self.engine
            .get_params()
            .to_json()
            .unwrap_or_else(|_| "{}".to_string())
    }

    pub fn all_notes_off(&mut self) {
        for note in 0..128u8 {
            self.engine
                .handle_midi(MidiMessage::NoteOff { channel: 0, note });
        }
    }

    /// Dispatch a raw 3-byte MIDI message. Status byte, data1, data2.
    /// This lets the JS Web MIDI layer forward messages verbatim.
    pub fn midi_message(&mut self, status: u8, data1: u8, data2: u8) {
        if let Some(msg) = MidiMessage::from_bytes(status, data1, data2) {
            self.engine.handle_midi(msg);
        }
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub fn create_engine() -> WasmEngine {
    WasmEngine::new()
}

#[wasm_bindgen]
pub fn get_param_count() -> u32 {
    36
}

#[wasm_bindgen]
pub fn get_param_name(param_id: u32) -> String {
    let param = ParameterId::from_raw(param_id);
    match param {
        ParameterId::Osc1Wave => "Osc1 Wave".to_string(),
        ParameterId::Osc1Freq => "Osc1 Frequency".to_string(),
        ParameterId::Osc1Fine => "Osc1 Fine Tune".to_string(),
        ParameterId::Osc1PulseWidth => "Osc1 Pulse Width".to_string(),
        ParameterId::Osc1PitchEnvDepth => "Osc1 Pitch Env".to_string(),
        ParameterId::Osc2Wave => "Osc2 Wave".to_string(),
        ParameterId::Osc2Coarse => "Osc2 Coarse".to_string(),
        ParameterId::Osc2Fine => "Osc2 Fine Tune".to_string(),
        ParameterId::Osc2PulseWidth => "Osc2 Pulse Width".to_string(),
        ParameterId::NoiseLevel => "Noise Level".to_string(),
        ParameterId::OscMix => "Osc Mix".to_string(),
        ParameterId::FilterCutoff => "Filter Cutoff".to_string(),
        ParameterId::FilterResonance => "Filter Resonance".to_string(),
        ParameterId::FilterEnvAmount => "Filter Env Amount".to_string(),
        ParameterId::FilterKeyTracking => "Filter Key Track".to_string(),
        ParameterId::AmpAttack => "Amp Attack".to_string(),
        ParameterId::AmpDecay => "Amp Decay".to_string(),
        ParameterId::AmpSustain => "Amp Sustain".to_string(),
        ParameterId::AmpRelease => "Amp Release".to_string(),
        ParameterId::FilterAttack => "Filter Attack".to_string(),
        ParameterId::FilterDecay => "Filter Decay".to_string(),
        ParameterId::FilterSustain => "Filter Sustain".to_string(),
        ParameterId::FilterRelease => "Filter Release".to_string(),
        ParameterId::LfoRate => "LFO Rate".to_string(),
        ParameterId::LfoWave => "LFO Wave".to_string(),
        ParameterId::LfoDepth => "LFO Depth".to_string(),
        ParameterId::LfoToOsc => "LFO to Osc".to_string(),
        ParameterId::LfoToFilter => "LFO to Filter".to_string(),
        ParameterId::LfoToPw => "LFO to PW".to_string(),
        ParameterId::PolyModOsc2ToOsc1Freq => "PolyMod Osc2>Freq".to_string(),
        ParameterId::PolyModOsc2ToOsc1Pw => "PolyMod Osc2>PW".to_string(),
        ParameterId::PolyModFilterEnvToFilter => "PolyMod Env>Filter".to_string(),
        ParameterId::Slop => "Slop".to_string(),
        ParameterId::UnisonVoices => "Unison Voices".to_string(),
        ParameterId::UnisonDetune => "Unison Detune".to_string(),
        ParameterId::MasterVolume => "Master Volume".to_string(),
    }
}
