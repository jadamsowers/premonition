#![no_std]
#![warn(missing_docs)]

//! Premonition Core DSP Engine
//!
//! A Prophet-style polyphonic analog synthesizer with authentic imperfections,
//! per-voice variations, and realistic signal path modeling.
//!
//! ## Architecture
//!
//! - Voice-based polyphony (5 or 8 voices)
//! - Dual VCOs with drift and analog imperfections
//! - Curtis/SSM-style 4-pole low-pass filter
//! - Global LFO and Poly-Mod section
//! - Optional effects (chorus, delay, reverb)

extern crate alloc;

mod audio;
mod control;
mod effects;

pub mod prelude;

use alloc::boxed::Box;
use audio::Engine;
use control::{MidiMessage, ParameterId, Parameters};

#[no_mangle]
pub extern "C" fn premonition_create() -> *mut Engine {
    let engine = Engine::new();
    Box::into_raw(Box::new(engine))
}

#[no_mangle]
pub extern "C" fn premonition_destroy(engine: *mut Engine) {
    if !engine.is_null() {
        unsafe { Box::from_raw(engine) };
    }
}

#[no_mangle]
pub extern "C" fn premonition_init(engine: *mut Engine, sample_rate: f32) {
    if engine.is_null() {
        return;
    }
    unsafe {
        (*engine).init(sample_rate);
    }
}

#[no_mangle]
pub extern "C" fn premonition_process(
    engine: *mut Engine,
    output_l: *mut f32,
    output_r: *mut f32,
    num_samples: usize,
    sample_rate: f32,
) {
    if engine.is_null() || output_l.is_null() || output_r.is_null() {
        return;
    }

    unsafe {
        let left = core::slice::from_raw_parts_mut(output_l, num_samples);
        let right = core::slice::from_raw_parts_mut(output_r, num_samples);
        (*engine).process(left, right, num_samples, sample_rate);
    }
}

#[no_mangle]
pub extern "C" fn premonition_note_on(engine: *mut Engine, channel: u8, note: u8, velocity: u8) {
    if engine.is_null() {
        return;
    }
    let msg = MidiMessage::NoteOn {
        channel,
        note,
        velocity,
    };
    unsafe {
        (*engine).handle_midi(msg);
    }
}

#[no_mangle]
pub extern "C" fn premonition_note_off(engine: *mut Engine, channel: u8, note: u8) {
    if engine.is_null() {
        return;
    }
    let msg = MidiMessage::NoteOff { channel, note };
    unsafe {
        (*engine).handle_midi(msg);
    }
}

#[no_mangle]
pub extern "C" fn premonition_set_param(engine: *mut Engine, param_id: u32, value: f32) {
    if engine.is_null() {
        return;
    }
    let param = ParameterId::from_raw(param_id);
    unsafe {
        (*engine).set_parameter(param, value);
    }
}

#[no_mangle]
pub extern "C" fn premonition_get_param(engine: *mut Engine, param_id: u32) -> f32 {
    if engine.is_null() {
        return 0.0;
    }
    let param = ParameterId::from_raw(param_id);
    unsafe { (*engine).get_parameter(param) }
}

#[no_mangle]
pub extern "C" fn premonition_panic() {
    panic!("Premonition engine panic");
}
