//! Premonition CLI - Standalone synthesizer application.

use clap::Parser;
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

fn run_interactive(state: &Arc<Mutex<SynthState>>, sample_rate: usize, buffer_size: usize) {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

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
        cpal::SampleFormat::F32 => {
            let state_inner = state_clone;
            device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut state = state_inner.lock().unwrap();
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
            )
        }
        _ => panic!("Unsupported sample format"),
    }
    .unwrap();

    stream.play().expect("Failed to start stream");

    println!("Audio stream started. Press Ctrl+C to exit...");

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

mod ctrlc {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    static RUNNING: AtomicBool = AtomicBool::new(true);

    pub fn set_handler<F>(mut handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut() + Send + 'static,
    {
        let running = Arc::new(RUNNING);
        let r = running.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 1];
            while r.load(Ordering::SeqCst) {
                if let Ok(()) = std::io::stdin().read(&mut buf) {
                    if buf[0] == 3 {
                        handler();
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(())
    }
}

impl std::io::Read for std::io::Stdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}
