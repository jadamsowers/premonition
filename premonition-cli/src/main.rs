//! Premonition CLI - Standalone synthesizer application.

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::MidiInput;
use premonition_core::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};
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

    #[arg(short = 'm', long)]
    midi_input: Option<String>,
}

struct SynthState {
    engine: Engine,
    note_held: bool,
    current_note: AtomicU8,
    mod_wheel: AtomicU8,
}

fn main() {
    let args = Args::parse();

    let state = Arc::new(Mutex::new(SynthState {
        engine: Engine::new(),
        note_held: false,
        current_note: AtomicU8::new(60),
        mod_wheel: AtomicU8::new(0),
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

    if let Some(ref midi_name) = args.midi_input {
        if let Err(e) = setup_midi(&state, midi_name) {
            eprintln!("Failed to setup MIDI: {}", e);
        }
    } else {
        match try_setup_midi(&state) {
            Ok(_) => println!("MIDI input connected!"),
            Err(e) => eprintln!(
                "No MIDI input found: {}. Connect a MIDI device and restart, or use -m to specify.",
                e
            ),
        }
    }

    println!();
    println!("MIDI Controls:");
    println!("  Note On/Off - Play notes");
    println!("  CC 1 - Mod wheel");
    println!("  CC 64 - Sustain");
    println!("  Pitch bend - Built-in");
    println!();
    println!("Starting audio... (will timeout in 60 seconds)");
    println!("Press Ctrl+C to exit.");
    println!();

    run_audio(&state, args.sample_rate);
}

fn try_setup_midi(state: &Arc<Mutex<SynthState>>) -> Result<(), Box<dyn std::error::Error>> {
    let midi_in = MidiInput::new("premonition")?;
    let ports = midi_in.ports();

    if ports.is_empty() {
        return Err("No MIDI input devices found".into());
    }

    let port_name = midi_in.port_name(&ports[0])?;
    println!("Using MIDI input: {}", port_name);

    let state_clone = state.clone();
    let _conn = midi_in.connect(
        &ports[0],
        "premonition-input",
        move |_timestamp, message, _| {
            if message.len() >= 3 {
                let status = message[0];
                let data1 = message[1];
                let data2 = message[2];

                let message_type = status & 0xF0;
                let channel = status & 0x0F;

                let mut state = state_clone.lock().unwrap();

                match message_type {
                    0x90 => {
                        if data2 > 0 {
                            state.engine.handle_midi(MidiMessage::NoteOn {
                                channel,
                                note: data1,
                                velocity: data2,
                            });
                            state.current_note.store(data1, Ordering::SeqCst);
                            state.note_held = true;
                        } else {
                            state.engine.handle_midi(MidiMessage::NoteOff {
                                channel,
                                note: data1,
                            });
                            state.note_held = false;
                        }
                    }
                    0x80 => {
                        state.engine.handle_midi(MidiMessage::NoteOff {
                            channel,
                            note: data1,
                        });
                        state.note_held = false;
                    }
                    0xB0 => match data1 {
                        1 => {
                            state.mod_wheel.store(data2, Ordering::SeqCst);
                            state.engine.handle_midi(MidiMessage::ModWheel {
                                channel,
                                value: data2 as f32 / 127.0,
                            });
                        }
                        64 => {
                            if data2 >= 64 {
                                state.engine.handle_midi(MidiMessage::SustainOn { channel });
                            } else {
                                state
                                    .engine
                                    .handle_midi(MidiMessage::SustainOff { channel });
                            }
                        }
                        123 | 120 => {
                            state
                                .engine
                                .handle_midi(MidiMessage::AllNotesOff { channel });
                        }
                        _ => {}
                    },
                    0xE0 => {
                        let bend_value = ((data2 as u16) << 7) | (data1 as u16);
                        let normalized = (bend_value as f32 - 8192.0) / 8192.0;
                        state.engine.handle_midi(MidiMessage::PitchBend {
                            channel,
                            value: normalized,
                        });
                    }
                    0xD0 => {
                        let current_note = state.current_note.load(Ordering::SeqCst);
                        state.engine.handle_midi(MidiMessage::Aftertouch {
                            channel,
                            note: current_note,
                            value: data1 as f32 / 127.0,
                        });
                    }
                    _ => {}
                }
            }
        },
        (),
    )?;

    Ok(())
}

fn setup_midi(
    state: &Arc<Mutex<SynthState>>,
    port_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let midi_in = MidiInput::new("premonition")?;
    let ports = midi_in.ports();

    let mut port_idx = None;
    for (i, p) in ports.iter().enumerate() {
        if midi_in.port_name(p)?.contains(port_name) {
            port_idx = Some(i);
            break;
        }
    }

    let port_idx = port_idx.ok_or_else(|| format!("MIDI port '{}' not found", port_name))?;

    let state_clone = state.clone();
    let _conn = midi_in.connect(
        &ports[port_idx],
        "premonition-input",
        move |_timestamp, message, _| {
            if message.len() >= 3 {
                let status = message[0];
                let data1 = message[1];
                let data2 = message[2];

                let message_type = status & 0xF0;
                let channel = status & 0x0F;

                let mut state = state_clone.lock().unwrap();

                match message_type {
                    0x90 => {
                        if data2 > 0 {
                            state.engine.handle_midi(MidiMessage::NoteOn {
                                channel,
                                note: data1,
                                velocity: data2,
                            });
                            state.current_note.store(data1, Ordering::SeqCst);
                            state.note_held = true;
                        } else {
                            state.engine.handle_midi(MidiMessage::NoteOff {
                                channel,
                                note: data1,
                            });
                            state.note_held = false;
                        }
                    }
                    0x80 => {
                        state.engine.handle_midi(MidiMessage::NoteOff {
                            channel,
                            note: data1,
                        });
                        state.note_held = false;
                    }
                    0xB0 => match data1 {
                        1 => {
                            state.mod_wheel.store(data2, Ordering::SeqCst);
                            state.engine.handle_midi(MidiMessage::ModWheel {
                                channel,
                                value: data2 as f32 / 127.0,
                            });
                        }
                        64 => {
                            if data2 >= 64 {
                                state.engine.handle_midi(MidiMessage::SustainOn { channel });
                            } else {
                                state
                                    .engine
                                    .handle_midi(MidiMessage::SustainOff { channel });
                            }
                        }
                        123 | 120 => {
                            state
                                .engine
                                .handle_midi(MidiMessage::AllNotesOff { channel });
                        }
                        _ => {}
                    },
                    0xE0 => {
                        let bend_value = ((data2 as u16) << 7) | (data1 as u16);
                        let normalized = (bend_value as f32 - 8192.0) / 8192.0;
                        state.engine.handle_midi(MidiMessage::PitchBend {
                            channel,
                            value: normalized,
                        });
                    }
                    0xD0 => {
                        let current_note = state.current_note.load(Ordering::SeqCst);
                        state.engine.handle_midi(MidiMessage::Aftertouch {
                            channel,
                            note: current_note,
                            value: data1 as f32 / 127.0,
                        });
                    }
                    _ => {}
                }
            }
        },
        (),
    )?;

    println!("Connected to MIDI input: {}", port_name);
    Ok(())
}

fn run_audio(state: &Arc<Mutex<SynthState>>, sample_rate: usize) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let config = device.default_output_config().unwrap();

    println!("Using audio device: {}", device.name().unwrap());

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

    std::thread::sleep(std::time::Duration::from_secs(60));

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
