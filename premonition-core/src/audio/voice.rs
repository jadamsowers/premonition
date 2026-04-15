//! Voice structure containing all per-voice state.

use crate::audio::{
    Envelope, Filter, Imperfection, Mixer, Noise, Oscillator, PolyMod, Vca, Waveform,
};
use crate::control::Parameters;

/// MIDI frequency table, pre-computed at compile time.
/// Generated from: 440.0 * 2^((n - 69) / 12) for n in 0..128
#[rustfmt::skip]
static MIDI_FREQ_TABLE: [f32; 128] = [
     8.176,   8.662,   9.177,   9.723,  10.301,  10.913,  11.562,  12.250,
    12.978,  13.750,  14.568,  15.434,  16.352,  17.324,  18.354,  19.445,
    20.602,  21.827,  23.125,  24.500,  25.957,  27.500,  29.135,  30.868,
    32.703,  34.648,  36.708,  38.891,  41.203,  43.654,  46.249,  48.999,
    51.913,  55.000,  58.270,  61.735,  65.406,  69.296,  73.416,  77.782,
    82.407,  87.307,  92.499,  97.999, 103.826, 110.000, 116.541, 123.471,
   130.813, 138.591, 146.832, 155.563, 164.814, 174.614, 184.997, 195.998,
   207.652, 220.000, 233.082, 246.942, 261.626, 277.183, 293.665, 311.127,
   329.628, 349.228, 369.994, 391.995, 415.305, 440.000, 466.164, 493.883,
   523.251, 554.365, 587.330, 622.254, 659.255, 698.456, 739.989, 783.991,
   830.609, 880.000, 932.328, 987.767,1046.502,1108.731,1174.659,1244.508,
  1318.510,1396.913,1479.978,1567.982,1661.219,1760.000,1864.655,1975.533,
  2093.005,2217.461,2349.318,2489.016,2637.020,2793.826,2959.955,3135.963,
  3322.438,3520.000,3729.310,3951.066,4186.009,4434.922,4698.636,4978.032,
  5274.041,5587.652,5919.911,6271.927,6644.875,7040.000,7458.620,7902.133,
  8372.018,8869.844,9397.273,9956.063,10548.082,11175.303,11839.822,12543.854,
];

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

        // Static pitch offsets (coarse/fine)
        let osc1_semitones = params.osc1_freq + params.osc1_fine;
        let mut osc2_semitones = params.osc2_coarse + params.osc2_fine;
        
        if params.osc2_lo_freq {
            // Drop Osc 2 by 5 octaves (60 semitones) to enter LFO range
            osc2_semitones -= 60.0;
        }
        
        // Fast approximation for 2^(semitones / 12)
        // 2^(x) ≈ 1.0 + 0.693*x + 0.240*x^2 for small x.
        // It's much faster than powf per sample.
        let osc1_static_factor = fast_exp2(osc1_semitones / 12.0);
        let osc2_static_factor = fast_exp2(osc2_semitones / 12.0);

        let lfo_to_pitch = params.lfo_to_osc * effective_lfo;
        let env_to_pitch = self.filter_env.value * params.osc1_pitch_env_depth;
        let poly_mod_osc_freq = 0.0; // TODO: properly hook up poly_mod

        let osc1_mod = osc1_static_factor * (1.0 + lfo_to_pitch + env_to_pitch + self.pitch_bend + poly_mod_osc_freq);
        let osc2_mod = osc2_static_factor * (1.0 + lfo_to_pitch + self.pitch_bend);

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

    #[inline(always)]
    fn note_to_freq(&self, note: u8) -> f32 {
        MIDI_FREQ_TABLE[note.min(127) as usize]
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

/// Fast 2^x approximation for pitch modulation.
/// Splits x into integer and fractional parts to avoid extreme errors
/// while remaining much faster than f32::powf()
#[inline(always)]
fn fast_exp2(x: f32) -> f32 {
    let int_part = x.floor();
    let frac_part = x - int_part;
    
    // 2^int_part is easily computed by manipulation of f32 bits if we wanted,
    // but a match or simple bit shift is fine. Or just rely on standard integer power:
    // (We cast to i32, then f32 or use the underlying standard function, but std isn't available?)
    // In our engine, semitones offset is typically small (e.g., -4.0 .. 4.0 for x)
    
    // f32 bit manipulation for 2^int_part:
    let i = int_part as i32;
    let pow2_int = f32::from_bits(((i + 127).max(0).min(255) as u32) << 23);

    // 2^frac_part ≈ 1.0 + 0.693147 * f + 0.240226 * f^2 + 0.0555 * f^3
    let f = frac_part;
    let pow2_frac = 1.0 + f * (0.693147 + f * (0.240226 + f * 0.0555));

    pow2_int * pow2_frac
}

