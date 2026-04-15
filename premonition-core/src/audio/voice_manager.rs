//! Voice allocation and management.

use crate::audio::Voice;
use crate::control::Parameters;

const MAX_VOICES: usize = 8;

pub struct VoiceManager {
    voices: [Voice; MAX_VOICES],
    current_voice: usize,
    unison_voices: usize,
}

impl VoiceManager {
    pub fn new(_num_voices: usize) -> Self {
        let voices = [
            Voice::new(0),
            Voice::new(1),
            Voice::new(2),
            Voice::new(3),
            Voice::new(4),
            Voice::new(5),
            Voice::new(6),
            Voice::new(7),
        ];
        Self {
            voices,
            current_voice: 0,
            unison_voices: 1,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        for voice in &mut self.voices {
            voice.init(sample_rate);
        }
    }

    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        let num_to_trigger = self.unison_voices.max(1);

        for i in 0..num_to_trigger {
            let voice_idx = (self.current_voice + i) % MAX_VOICES;
            self.voices[voice_idx].note_on(channel, note, velocity);
        }

        self.current_voice = (self.current_voice + num_to_trigger) % MAX_VOICES;
    }

    pub fn note_off(&mut self, channel: u8, note: u8) {
        for voice in &mut self.voices {
            if voice.active && voice.note == note && voice.channel == channel {
                voice.note_off();
            }
        }
    }

    pub fn process(
        &mut self,
        params: &Parameters,
        lfo_value: f32,
        stereo_spread: f32,
        output_l: &mut f32,
        output_r: &mut f32,
    ) {
        let mut combined_l = 0.0f32;
        let mut combined_r = 0.0f32;

        for voice in &mut self.voices {
            let mut voice_l = 0.0f32;
            let mut voice_r = 0.0f32;

            voice.process(params, lfo_value, stereo_spread, &mut voice_l, &mut voice_r);

            if voice.is_active() {
                combined_l += voice_l;
                combined_r += voice_r;
            }
        }

        *output_l = combined_l;
        *output_r = combined_r;
    }

    pub fn update_params(&mut self, params: &Parameters) {
        self.unison_voices = params.unison_voices as usize;

        for voice in &mut self.voices {
            voice.osc1.update_params(params);
            voice.osc2.update_params(params);
            voice.filter.update_params(params);
            voice.amp_env.update_params(params);
            voice.filter_env.update_params(params);
        }
    }

    pub fn set_pitch_bend(&mut self, channel: u8, value: f32) {
        for voice in &mut self.voices {
            if voice.channel == channel {
                voice.set_pitch_bend(value);
            }
        }
    }

    pub fn set_mod_wheel(&mut self, channel: u8, value: f32) {
        for voice in &mut self.voices {
            if voice.channel == channel {
                voice.set_mod_wheel(value);
            }
        }
    }

    pub fn set_aftertouch(&mut self, channel: u8, note: u8, value: f32) {
        for voice in &mut self.voices {
            if voice.channel == channel && (voice.note == note || note == 128) {
                voice.set_aftertouch(value);
            }
        }
    }

    pub fn set_sustain(&mut self, channel: u8, enabled: bool) {
        for voice in &mut self.voices {
            if voice.channel == channel {
                voice.set_sustain(enabled);
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.active = false;
            voice.amp_env.release();
            voice.filter_env.release();
        }
    }

    #[allow(dead_code)]
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }
}
