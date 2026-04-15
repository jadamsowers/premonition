//! Main audio processing engine.

use crate::audio::{EffectsChain, Lfo, OutputStage, VoiceManager};
use crate::control::{MidiMessage, ParameterId, Parameters};

const MAX_VOICES: usize = 8;
const STEREO_SPREAD: f32 = 0.3;

pub struct Engine {
    sample_rate: f32,
    voice_manager: VoiceManager,
    output: OutputStage,
    lfo: Lfo,
    effects: EffectsChain,
    params: Parameters,
    notes_held: u8,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            voice_manager: VoiceManager::new(MAX_VOICES),
            output: OutputStage::new(MAX_VOICES),
            lfo: Lfo::new(),
            effects: EffectsChain::new(),
            params: Parameters::default(),
            notes_held: 0,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.voice_manager.init(sample_rate);
        self.voice_manager.update_params(&self.params);
        self.output.init(sample_rate);
        self.lfo.init(sample_rate);
        self.lfo.update_params(&self.params);
        self.effects.init(sample_rate);
    }

    pub fn process(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        num_samples: usize,
        sample_rate: f32,
    ) {
        if self.sample_rate != sample_rate {
            self.init(sample_rate);
        }

        let stereo_spread = if self.params.unison_voices > 1 {
            STEREO_SPREAD
        } else {
            0.0
        };

        for i in 0..num_samples {
            let lfo_value = self.lfo.process(&self.params);
            let (mut voice_out_l, mut voice_out_r) = (0.0f32, 0.0f32);

            self.voice_manager.process(
                &self.params,
                lfo_value,
                stereo_spread,
                &mut voice_out_l,
                &mut voice_out_r,
            );

            voice_out_l = self.effects.process_sample(voice_out_l);
            voice_out_r = self.effects.process_sample(voice_out_r);

            let master_vol = self.params.master_volume;
            left[i] = voice_out_l * master_vol;
            right[i] = voice_out_r * master_vol;
        }
    }

    pub fn handle_midi(&mut self, msg: MidiMessage) {
        match msg {
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
            } => {
                if velocity > 0 {
                    self.voice_manager.note_on(channel, note, velocity);
                    self.notes_held += 1;
                } else {
                    self.voice_manager.note_off(channel, note);
                    self.notes_held = self.notes_held.saturating_sub(1);
                }
            }
            MidiMessage::NoteOff { channel, note } => {
                self.voice_manager.note_off(channel, note);
                self.notes_held = self.notes_held.saturating_sub(1);
            }
            MidiMessage::PitchBend { channel, value } => {
                self.voice_manager.set_pitch_bend(channel, value);
            }
            MidiMessage::ModWheel { channel, value } => {
                self.voice_manager.set_mod_wheel(channel, value);
            }
            MidiMessage::SustainOn { channel } => {
                self.voice_manager.set_sustain(channel, true);
            }
            MidiMessage::SustainOff { channel } => {
                self.voice_manager.set_sustain(channel, false);
            }
            MidiMessage::Aftertouch {
                channel,
                note,
                value,
            } => {
                self.voice_manager.set_aftertouch(channel, note, value);
            }
            MidiMessage::AllNotesOff { channel: _ } => {
                self.voice_manager.all_notes_off();
            }
            MidiMessage::ControlChange {
                channel: _,
                controller,
                value,
            } => {
                let norm = value as f32 / 127.0;
                // Standard GM / Prophet-inspired CC map
                // All unmapped CCs are silently ignored.
                let param = match controller {
                    // ── Mixer / Output ────────────────────────────────────
                    7  => Some(ParameterId::MasterVolume),
                    10 => Some(ParameterId::OscMix),
                    15 => Some(ParameterId::NoiseLevel),
                    // ── Filter ───────────────────────────────────────────
                    74 => Some(ParameterId::FilterCutoff),
                    71 => Some(ParameterId::FilterResonance),
                    16 => Some(ParameterId::FilterEnvAmount),
                    14 => Some(ParameterId::FilterKeyTracking),
                    // ── Amp Envelope ─────────────────────────────────────
                    73 => Some(ParameterId::AmpAttack),
                    75 => Some(ParameterId::AmpDecay),
                    76 => Some(ParameterId::AmpSustain),
                    72 => Some(ParameterId::AmpRelease),
                    // ── Filter Envelope ───────────────────────────────────
                    77 => Some(ParameterId::FilterAttack),
                    78 => Some(ParameterId::FilterDecay),
                    79 => Some(ParameterId::FilterSustain),
                    80 => Some(ParameterId::FilterRelease),
                    // ── LFO ──────────────────────────────────────────────
                    17 => Some(ParameterId::LfoRate),
                    // CC 1 = mod wheel → already handled as ModWheel above,
                    // but also accepted as LfoDepth via generic CC path
                    18 => Some(ParameterId::LfoDepth),
                    24 => Some(ParameterId::LfoToOsc),
                    25 => Some(ParameterId::LfoToFilter),
                    26 => Some(ParameterId::LfoToPw),
                    // ── Oscillators ───────────────────────────────────────
                    20 => Some(ParameterId::Osc1Fine),
                    21 => Some(ParameterId::Osc1PulseWidth),
                    22 => Some(ParameterId::Osc2Coarse),
                    23 => Some(ParameterId::Osc2Fine),
                    27 => Some(ParameterId::Osc2PulseWidth),
                    // ── Poly Mod ─────────────────────────────────────────
                    28 => Some(ParameterId::PolyModOsc2ToOsc1Freq),
                    29 => Some(ParameterId::PolyModOsc2ToOsc1Pw),
                    30 => Some(ParameterId::PolyModFilterEnvToFilter),
                    // ── Character ────────────────────────────────────────
                    19 => Some(ParameterId::Slop),
                    31 => Some(ParameterId::UnisonDetune),
                    _ => None,
                };
                if let Some(p) = param {
                    self.set_parameter(p, norm);
                }
            }
        }
    }

    pub fn set_parameter(&mut self, param: ParameterId, value: f32) {
        self.params.set(param, value);
        self.voice_manager.update_params(&self.params);
        self.lfo.update_params(&self.params);
    }

    pub fn get_parameter(&self, param: ParameterId) -> f32 {
        self.params.get(param)
    }

    pub fn get_params(&self) -> &Parameters {
        &self.params
    }

    pub fn get_params_mut(&mut self) -> &mut Parameters {
        &mut self.params
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
