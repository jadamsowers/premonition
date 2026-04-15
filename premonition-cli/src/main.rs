//! Premonition CLI - Standalone synthesizer application.

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use premonition_core::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(name = "premonition")]
#[command(about = "Prophet-style analog synthesizer", long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 44100)]
    sample_rate: usize,

    #[arg(short, long, default_value_t = 512)]
    buffer_size: usize,

    #[arg(short, long)]
    preset: Option<String>,
}

struct SynthState {
    engine: Engine,
    note_on: bool,
    current_note: u8,
}

fn main() {
    let args = Args::parse();

    let state = Arc::new(Mutex::new(SynthState {
        engine: Engine::new(),
        note_on: false,
        current_note: 60,
    }));

    {
        let mut state_guard = state.lock().unwrap();
        state_guard.engine.init(args.sample_rate as f32);

        if let Some(preset_path) = &args.preset {
            load_preset(&mut state_guard.engine, preset_path);
        }
    }

    println!("Premonition Synthesizer");
    println!("======================");
    println!("Sample Rate: {} Hz", args.sample_rate);
    println!("Buffer Size: {} samples", args.buffer_size);
    println!();
    println!("Controls:");
    println!("  Z/X - Note -/+");
    println!("  Space - Hold note");
    println!("  Q/W/E/R/T - Filter cutoff/resonance/env");
    println!("  A/S/D/F/G/H - Amp attack/decay/sustain/release");
    println!("  P - Print current parameters");
    println!("  Esc - Exit");
    println!();

    run_interactive(&state, args.sample_rate, args.buffer_size);
}

fn run_interactive(state: &Arc<Mutex<SynthState>>, sample_rate: usize, _buffer_size: usize) {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(60));
        r.store(false, Ordering::SeqCst);
    });

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let config = device.default_output_config().unwrap();

    println!("Using audio device: {}", device.name().unwrap());
    println!("Starting audio thread...");

    let state_clone = state.clone();

    let err_fn = |err| eprintln!("Error in audio thread: {}", err);

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut state = state_clone.lock().unwrap();
                let stereo = data.chunks_mut(2);

                for frame in stereo {
                    let (mut left, mut right) = ([0.0f32], [0.0f32]);
                    state
                        .engine
                        .process(&mut left, &mut right, 1, sample_rate as f32);

                    if frame.len() >= 2 {
                        frame[0] = left[0];
                        frame[1] = right[0];
                    }
                }
            },
            err_fn,
            None,
        ),
        _ => panic!("Unsupported sample format"),
    }
    .unwrap();

    stream.play().expect("Failed to start stream");

    println!("Audio stream started. Press Ctrl+C to exit or wait 60 seconds...");

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("Shutting down...");
}

fn load_preset(engine: &mut Engine, path: &str) {
    match std::fs::read_to_string(path) {
        Ok(json) => match Parameters::from_json(&json) {
            Ok(params) => {
                let engine_params = engine.get_params_mut();
                *engine_params = params;
                println!("Loaded preset: {}", path);
            }
            Err(e) => {
                eprintln!("Failed to parse preset: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Failed to read preset file: {}", e);
        }
    }
}
