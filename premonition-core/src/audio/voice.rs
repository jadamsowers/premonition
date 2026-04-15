//! Voice structure containing all per-voice state.

use crate::audio::{
    Envelope, Filter, Imperfection, Lfo, Mixer, Noise, Oscillator, PolyMod, Vca, Waveform,
};
use crate::control::Parameters;

fn u8_to_waveform(value: u8) -> Waveform {
    match value {
        0 => Waveform::Saw,
        1 => Waveform::Pulse,
        2 => Waveform::Triangle,
        _ => Waveform::Saw,
    }
}

const VELOCITY_TO_ATTENUATION: f32 = 0.00315;

pub struct Voice {
    pub active: bool,
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,

    pub osc1: Oscillator,
    pub osc2: Oscillator,
    pub noise: Noise,
    pub mixer: Mixer,
    pub filter: Filter,
    pub vca: Vca,
    pub amp_env: Envelope,
    pub filter_env: Envelope,
    pub poly_mod: PolyMod,

    pub imperfection: Imperfection,

    pub output_l: f32,
    pub output_r: f32,

    pub pitch_bend: f32,
    pub mod_wheel: f32,
    pub aftertouch: f32,
    pub sustain: bool,

    pub lfo_delay_counter: f32,
}

impl Voice {
    pub fn new(voice_index: usize) -> Self {
        Self {
            active: false,
            note: 60,
            velocity: 100,
            channel: 0,
            osc1: Oscillator::new(voice_index),
            osc2: Oscillator::new(voice_index + 100),
            noise: Noise::new(voice_index),
            mixer: Mixer::new(voice_index),
            filter: Filter::new(voice_index),
            vca: Vca::new(voice_index),
            amp_env: Envelope::new(voice_index),
            filter_env: Envelope::new(voice_index + 50),
            poly_mod: PolyMod::new(voice_index),
            imperfection: Imperfection::new(voice_index),
            output_l: 0.0,
            output_r: 0.0,
            pitch_bend: 0.0,
            mod_wheel: 0.0,
            aftertouch: 0.0,
            sustain: false,
            lfo_delay_counter: 0.0,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.osc1.init(sample_rate);
        self.osc2.init(sample_rate);
        self.noise.init(sample_rate);
        self.mixer.init(sample_rate);
        self.filter.init(sample_rate);
        self.vca.init(sample_rate);
        self.amp_env.init(sample_rate);
        self.filter_env.init(sample_rate);
        self.poly_mod.init(sample_rate);
        self.imperfection.init();
    }

    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        self.active = true;
        self.note = note;
        self.velocity = velocity;
        self.channel = channel;

        let base_freq = self.note_to_freq(note);
        self.osc1.set_frequency(base_freq);
        self.osc2.set_frequency(base_freq);

        let velocity_float = velocity as f32 / 127.0;
        let velocity_atten = 1.0 - (velocity_float * VELOCITY_TO_ATTENUATION * 127.0).min(1.0);
        self.vca.set_velocity_sensitivity(velocity_atten);

        self.amp_env.trigger();
        self.filter_env.trigger();
    }

    pub fn note_off(&mut self) {
        if !self.sustain {
            self.active = false;
            self.amp_env.release();
            self.filter_env.release();
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
        if !self.active && self.amp_env.is_idle() {
            *output_l = 0.0;
            *output_r = 0.0;
            return;
        }

        self.lfo_delay_counter = (self.lfo_delay_counter + 1.0).min(1000.0);
        let lfo_delay = (self.lfo_delay_counter / 1000.0).min(1.0);
        let effective_lfo = lfo_value * lfo_delay;

        let lfo_to_pitch = params.lfo_to_osc * effective_lfo;
        let env_to_pitch = self.filter_env.value * params.osc1_pitch_env_depth;

        let osc1_mod = 1.0 + lfo_to_pitch + env_to_pitch + self.pitch_bend;
        let osc2_mod = 1.0 + lfo_to_pitch + self.pitch_bend;

        self.osc1.set_pitch_mod(osc1_mod);
        self.osc2.set_pitch_mod(osc2_mod);

        let osc1_pw = params.osc1_pulse_width + (effective_lfo * params.lfo_to_pw);
        let osc2_pw = params.osc2_pulse_width + (effective_lfo * params.lfo_to_pw);
        self.osc1.set_pulse_width(osc1_pw);
        self.osc2.set_pulse_width(osc2_pw);

        let osc1_out = self.osc1.process(u8_to_waveform(params.osc1_wave));
        let osc2_out = self.osc2.process(u8_to_waveform(params.osc2_wave));
        let noise_out = self.noise.process(params.noise_level);

        let (mix_out, _) = self.mixer.process(osc1_out, osc2_out, noise_out, params);

        let filter_env_amount =
            params.filter_env_amount * (1.0 + self.imperfection.filter_env_offset);
        let filter_cutoff = params.filter_cutoff + (self.filter_env.value * filter_env_amount);
        let filter_mod = effective_lfo * params.lfo_to_filter;

        self.filter.set_cutoff(filter_cutoff + filter_mod);
        self.filter.set_resonance(params.filter_resonance);
        self.filter
            .set_keyboard_tracking(params.filter_keyboard_tracking);

        let poly_mod_filter_mod = self.poly_mod.get_filter_mod();
        let filter_input = mix_out + poly_mod_filter_mod;

        let filter_out = self.filter.process(filter_input);

        let vca_level = self.amp_env.value * self.vca.get_gain();
        let vca_out = self.vca.process(filter_out, vca_level);

        let spread = stereo_spread * self.imperfection.pan_position;
        self.output_l = vca_out * (1.0 - spread);
        self.output_r = vca_out * (1.0 + spread);

        *output_l = self.output_l;
        *output_r = self.output_r;

        self.amp_env.process(params);
        self.filter_env.process(params);
        self.imperfection.update();
    }

    pub fn is_active(&self) -> bool {
        self.active || !self.amp_env.is_idle()
    }

    fn note_to_freq(&self, note: u8) -> f32 {
        440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    pub fn set_pitch_bend(&mut self, bend: f32) {
        self.pitch_bend = bend;
    }

    pub fn set_mod_wheel(&mut self, value: f32) {
        self.mod_wheel = value;
    }

    pub fn set_aftertouch(&mut self, value: f32) {
        self.aftertouch = value;
    }

    pub fn set_sustain(&mut self, enabled: bool) {
        self.sustain = enabled;
        if !enabled && !self.active {
            self.amp_env.release();
            self.filter_env.release();
        }
    }
}
