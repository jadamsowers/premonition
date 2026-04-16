//! Premonition CLI - Standalone synthesizer application.

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::MidiInput;
use premonition_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

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

    #[arg(short = 'a', long)]
    audio_output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = CLIConfig::load();

    let state = Arc::new(Mutex::new(SynthState::new()));

    {
        let mut state_guard = state.lock().unwrap();
        state_guard.engine.init(args.sample_rate as f32);

        // Load preset if provided
        if let Some(ref preset_name) = args.preset {
            if let Err(e) = load_preset(&mut state_guard.engine, preset_name) {
                state_guard.midi_status = format!("Preset error: {}", e);
            }
        }
    }

    // Capture MIDI connection
    let mut _midi_conn = None;
    let midi_input = args.midi_input.or_else(|| config.midi_device.clone());

    if let Some(ref midi_name) = midi_input {
        match setup_midi_by_name(&state, midi_name) {
            Ok(conn) => {
                _midi_conn = Some(conn);
            }
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.midi_status = format!("Error: {}", e);
            }
        }
    } else {
        match interactive_midi_selection(&state) {
            Ok(conn) => {
                _midi_conn = Some(conn);
            }
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.midi_status = format!("Error: {}", e);
            }
        }
    }

    // Capture Audio Selection
    let audio_output = args.audio_output.or_else(|| config.audio_device.clone());
    let audio_device = if let Some(ref device_name) = audio_output {
        match setup_audio_by_name(&state, device_name) {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Audio device '{}' not found: {}. Falling back.", device_name, e);
                setup_default_audio(&state)?
            }
        }
    } else {
        match setup_default_audio(&state) {
            Ok(device) => device,
            Err(_) => {
                println!("No default audio device found. Falling back to selection.");
                interactive_audio_selection(&state)?
            }
        }
    };

    // Save configuration
    {
        let s = state.lock().unwrap();
        let mut updated_config = config;
        // Strip status prefixes if present
        updated_config.midi_device = Some(s.midi_status.replace("Connected to ", ""));
        updated_config.audio_device = Some(s.audio_status.replace("Connected to ", ""));
        let _ = updated_config.save();
    }

    println!("\x1b[2J\x1b[H");
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              PREMONITION SYNTHESIZER                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "Sample Rate: {} Hz, Buffer: {} samples",
        args.sample_rate, args.buffer_size
    );
    println!();

    {
        let s = state.lock().unwrap();
        println!("MIDI:  {}", s.midi_status);
        println!("Audio: {}", s.audio_status);
    }

    println!();
    println!("Starting audio stream...");
    println!("(Press 'Q' to quit, 'S' to save current preset, Home row keys to play)");

    run_audio(&state, audio_device, args.sample_rate);
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct CLIConfig {
    midi_device: Option<String>,
    audio_device: Option<String>,
}

impl CLIConfig {
    fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".premonition_config.json")
    }

    fn load() -> Self {
        fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> io::Result<()> {
        let s = serde_json::to_string_pretty(self).unwrap();
        fs::write(Self::path(), s)
    }
}

fn load_preset(engine: &mut Engine, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from("premonition-cli/assets/presets").join(name);
    if path.extension().is_none() {
        path.set_extension("json");
    }

    let json = fs::read_to_string(path)?;
    let params: Parameters = serde_json::from_str(&json)?;
    *engine.get_params_mut() = params;
    engine.init(44100.0); // Simple re-init, sample rate will be fixed in next process call
    Ok(())
}

fn save_preset(engine: &Engine, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from("premonition-cli/assets/presets").join(name);
    if path.extension().is_none() {
        path.set_extension("json");
    }

    let json = serde_json::to_string_pretty(engine.get_params())?;
    fs::write(path, json)?;
    Ok(())
}

struct SynthState {
    engine: Engine,
    notes_held: Vec<u8>,
    viz_buffer: Vec<f32>,
    last_processed_sample: f32,
    midi_status: String,
    audio_status: String,
}

impl SynthState {
    fn new() -> Self {
        Self {
            engine: Engine::new(),
            notes_held: Vec::new(),
            viz_buffer: Vec::with_capacity(1024),
            last_processed_sample: 0.0,
            midi_status: "(No device connected)".to_string(),
            audio_status: "(No device connected)".to_string(),
        }
    }
}

fn handle_midi_event(state: &mut SynthState, message: &[u8]) {
    if message.len() >= 3 {
        let status = message[0];
        let data1 = message[1];
        let data2 = message[2];

        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            0x90 => {
                if data2 > 0 {
                    state.engine.handle_midi(MidiMessage::NoteOn {
                        channel,
                        note: data1,
                        velocity: data2,
                    });
                    if !state.notes_held.contains(&data1) {
                        state.notes_held.push(data1);
                    }
                    println!("\x1b[32mNote On:  {} (vel: {})\x1b[0m", data1, data2);
                } else {
                    state.engine.handle_midi(MidiMessage::NoteOff {
                        channel,
                        note: data1,
                    });
                    state.notes_held.retain(|&n| n != data1);
                    println!("\x1b[31mNote Off: {}\x1b[0m", data1);
                }
            }
            0x80 => {
                state.engine.handle_midi(MidiMessage::NoteOff {
                    channel,
                    note: data1,
                });
                state.notes_held.retain(|&n| n != data1);
                println!("\x1b[31mNote Off: {}\x1b[0m", data1);
            }
            0xB0 => match data1 {
                1 => {
                    state.engine.handle_midi(MidiMessage::ModWheel {
                        channel,
                        value: data2 as f32 / 127.0,
                    });
                    println!("\x1b[34mMod Wheel: {}\x1b[0m", data2);
                }
                64 => {
                    if data2 >= 64 {
                        state.engine.handle_midi(MidiMessage::SustainOn { channel });
                        println!("\x1b[36mSustain: ON\x1b[0m");
                    } else {
                        state
                            .engine
                            .handle_midi(MidiMessage::SustainOff { channel });
                        println!("\x1b[36mSustain: OFF\x1b[0m");
                    }
                }
                // Standard MIDI CC Mappings
                74 => { // Filter Cutoff
                    state.engine.set_parameter(ParameterId::FilterCutoff, data2 as f32 / 127.0);
                    println!("\x1b[34mCC Cutoff: {}\x1b[0m", data2);
                }
                71 => { // Filter Resonance
                    state.engine.set_parameter(ParameterId::FilterResonance, data2 as f32 / 127.0);
                    println!("\x1b[34mCC Resonance: {}\x1b[0m", data2);
                }
                73 => { // Attack
                    state.engine.set_parameter(ParameterId::AmpAttack, data2 as f32 / 127.0);
                    println!("\x1b[34mCC Attack: {}\x1b[0m", data2);
                }
                72 => { // Release
                    state.engine.set_parameter(ParameterId::AmpRelease, data2 as f32 / 127.0);
                    println!("\x1b[34mCC Release: {}\x1b[0m", data2);
                }
                123 | 120 => {
                    state
                        .engine
                        .handle_midi(MidiMessage::AllNotesOff { channel });
                    println!("\x1b[35mAll Notes Off\x1b[0m");
                }
                _ => {
                    println!("\x1b[90mCC {}: {}\x1b[0m", data1, data2);
                }
            },
            0xE0 => {
                let bend_value = ((data2 as u16) << 7) | (data1 as u16);
                let normalized = (bend_value as f32 - 8192.0) / 8192.0;
                state.engine.handle_midi(MidiMessage::PitchBend {
                    channel,
                    value: normalized,
                });
                println!("\x1b[33mPitch Bend: {}\x1b[0m", bend_value);
            }
            0xD0 => {
                state.engine.handle_midi(MidiMessage::Aftertouch {
                    channel,
                    note: data1,
                    value: data1 as f32 / 127.0,
                });
                println!("\x1b[35mAftertouch: {}\x1b[0m", data1);
            }
            _ => {
                println!("\x1b[90mOther MIDI: status={:02X} d1={:02X} d2={:02X}\x1b[0m", status, data1, data2);
            }
        }
    }
}

fn interactive_midi_selection(
    state: &Arc<Mutex<SynthState>>,
) -> Result<midir::MidiInputConnection<()>, Box<dyn std::error::Error>> {
    let midi_in = MidiInput::new("premonition")?;
    let ports = midi_in.ports();

    if ports.is_empty() {
        return Err("No MIDI input devices found".into());
    }

    if ports.len() == 1 {
        let port_name = midi_in.port_name(&ports[0])?;
        println!("Using only available MIDI device: {}", port_name);
        return setup_midi_by_index(state, 0);
    }

    println!("\nAvailable MIDI input devices:");
    for (i, p) in ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
    }

    print!("\nSelect MIDI device [0-{}]: ", ports.len() - 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse().map_err(|_| "Invalid selection")?;

    if selection >= ports.len() {
        return Err("Selection out of range".into());
    }

    setup_midi_by_index(state, selection)
}

fn setup_midi_by_index(
    state: &Arc<Mutex<SynthState>>,
    port_index: usize,
) -> Result<midir::MidiInputConnection<()>, Box<dyn std::error::Error>> {
    let midi_in = MidiInput::new("premonition")?;
    let ports = midi_in.ports();

    if ports.is_empty() {
        return Err("No MIDI ports available".into());
    }
    if port_index >= ports.len() {
        return Err("Invalid port index".into());
    }

    let port_name = midi_in.port_name(&ports[port_index])?;
    let state_clone = state.clone();

    let _conn = midi_in.connect(
        &ports[port_index],
        "premonition-input",
        move |_timestamp, message, _| {
            let mut state = state_clone.lock().unwrap();
            handle_midi_event(&mut state, message);
        },
        (),
    )?;

    state.lock().unwrap().midi_status = format!("Connected to {}", port_name);
    Ok(_conn)
}

fn setup_midi_by_name(
    state: &Arc<Mutex<SynthState>>,
    port_name: &str,
) -> Result<midir::MidiInputConnection<()>, Box<dyn std::error::Error>> {
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
            let mut state = state_clone.lock().unwrap();
            handle_midi_event(&mut state, message);
        },
        (),
    )?;

    state.lock().unwrap().midi_status = format!("Connected to {}", port_name);
    Ok(_conn)
}

fn setup_default_audio(state: &Arc<Mutex<SynthState>>) -> Result<cpal::Device, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("No default output device")?;
    let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
    state.lock().unwrap().audio_status = format!("Connected to {} (Default)", name);
    Ok(device)
}

fn setup_audio_by_name(
    state: &Arc<Mutex<SynthState>>,
    name: &str,
) -> Result<cpal::Device, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    for device in host.output_devices()? {
        if device.name()?.contains(name) {
            state.lock().unwrap().audio_status = format!("Connected to {}", device.name()?);
            return Ok(device);
        }
    }
    Err(format!("Audio device '{}' not found", name).into())
}

fn interactive_audio_selection(
    state: &Arc<Mutex<SynthState>>,
) -> Result<cpal::Device, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let devices: Vec<_> = host.output_devices()?.map(|d| d).collect();

    if devices.is_empty() {
        return Err("No audio output devices found".into());
    }

    println!("\nAvailable audio output devices:");
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {}", i, d.name().unwrap_or_else(|_| "Unknown Device".to_string()));
    }

    print!("\nSelect audio device [0-{}]: ", devices.len() - 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse().map_err(|_| "Invalid selection")?;

    if selection >= devices.len() {
        return Err("Selection out of range".into());
    }

    let device = devices.into_iter().nth(selection).unwrap();
    let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
    state.lock().unwrap().audio_status = format!("Connected to {}", name);
    Ok(device)
}

fn run_audio(state: &Arc<Mutex<SynthState>>, device: cpal::Device, sample_rate: usize) {
    let config = device.default_output_config().unwrap();
    let actual_sample_rate = config.sample_rate().0 as f32;
    let state_clone = state.clone();
    let err_fn = |err| eprintln!("Error in audio thread: {}", err);

    // Initialize engine with the real device sample rate
    {
        let mut mut_state = state.lock().unwrap();
        mut_state.engine.init(actual_sample_rate);
    }

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
                        .process(&mut left, &mut right, 1, actual_sample_rate);

                    if frame.len() >= 2 {
                        frame[0] = left[0];
                        frame[1] = right[0];
                    }

                    let sample = (left[0] + right[0]) * 0.5;
                    state.last_processed_sample = sample;
                    if state.viz_buffer.len() < 1600 {
                        state.viz_buffer.push(sample);
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
    
    // START INTERACTIVE LOOP
    interactive_loop(state);
}

fn interactive_loop(state: &Arc<Mutex<SynthState>>) {
    enable_raw_mode().unwrap();
    let mut last_viz = std::time::Instant::now();

    loop {
        // Handle visualization at ~10Hz
        if last_viz.elapsed().as_millis() > 100 {
            let mut state = state.lock().unwrap();
            draw_viz(&state);
            state.viz_buffer.clear();
            last_viz = std::time::Instant::now();
        }

        // Handle Keyboard Events
        if event::poll(std::time::Duration::from_millis(10)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('q') {
                    disable_raw_mode().unwrap();
                    println!("\nExiting Premonition...");
                    std::process::exit(0);
                }
                
                if key.code == KeyCode::Char('s') {
                    // Quick save current state to "recent.json"
                    let state = state.lock().unwrap();
                    if let Err(e) = save_preset(&state.engine, "recent") {
                        eprintln!("Save failed: {}", e);
                    } else {
                        println!("\nSaved current state to assets/presets/recent.json");
                    }
                    continue;
                }

                // Piano keys mapping (A=C4, W=C#4, S=D4, etc.)
                let note = match key.code {
                    KeyCode::Char('a') => Some(60), // C4
                    KeyCode::Char('w') => Some(61), // C#4
                    KeyCode::Char('s') => Some(62), // D4
                    KeyCode::Char('e') => Some(63), // D#4
                    KeyCode::Char('d') => Some(64), // E4
                    KeyCode::Char('f') => Some(65), // F4
                    KeyCode::Char('t') => Some(66), // F#4
                    KeyCode::Char('g') => Some(67), // G4
                    KeyCode::Char('y') => Some(68), // G#4
                    KeyCode::Char('h') => Some(69), // A4
                    KeyCode::Char('u') => Some(70), // A#4
                    KeyCode::Char('j') => Some(71), // B4
                    KeyCode::Char('k') => Some(72), // C5
                    _ => None,
                };

                if let Some(n) = note {
                    let mut s = state.lock().unwrap();
                    // Basic note on/off toggle logic for computer keyboard
                    if s.notes_held.contains(&n) {
                        s.engine.handle_midi(MidiMessage::NoteOff { channel: 0, note: n });
                        s.notes_held.retain(|&x| x != n);
                    } else {
                        s.engine.handle_midi(MidiMessage::NoteOn { channel: 0, note: n, velocity: 100 });
                        s.notes_held.push(n);
                    }
                }
            }
        }
    }
}

fn draw_viz(state: &SynthState) {
    // MOVE CURSOR TO A FIXED POSITION (under the header)
    print!("\x1b[8;1H");

    // Level Meter
    let level = state.last_processed_sample.abs().min(1.0);
    let meter_width = 40;
    let filled = (level * meter_width as f32) as usize;
    let empty = meter_width - filled;
    print!("Level: [{}{}] {:.2}  ", "█".repeat(filled), " ".repeat(empty), level);

    // Metadata Status
    print!("\nMIDI:  {}          ", state.midi_status);
    print!("\nAudio: {}          ", state.audio_status);

    // Active Notes
    print!("\nNotes: ");
    if state.notes_held.is_empty() {
        print!("None             ");
    } else {
        for note in &state.notes_held {
            print!("{} ", note);
        }
        print!("                ");
    }

    // High-Resolution 2D Braille Waveform
    let char_height = 4;
    let sub_height = char_height * 4;
    let display_points = 160;
    let mut grid = vec![vec![0u8; display_points / 2]; char_height];

    // Downsample for zoomed out view
    let stride = (state.viz_buffer.len() as f32 / display_points as f32).max(1.0) as usize;

    for i in 0..display_points {
        let buffer_idx = i * stride;
        if buffer_idx >= state.viz_buffer.len() { break; }
        
        let sample = state.viz_buffer[buffer_idx];
        let col = i / 2;
        let sub_col = i % 2;
        
        // Normalize sample to sub-pixel height
        let normalized = (sample + 1.0) * 0.5;
        let sub_row = ((1.0 - normalized) * (sub_height as f32 - 1.0)) as usize;
        let sub_row = sub_row.min(sub_height - 1);
        
        let char_row = sub_row / 4;
        let dot_row = sub_row % 4;

        // Braille dot mapping
        let bit = match (sub_col, dot_row) {
            (0, 0) => 0x01,
            (0, 1) => 0x02,
            (0, 2) => 0x04,
            (0, 3) => 0x40,
            (1, 0) => 0x08,
            (1, 1) => 0x10,
            (1, 2) => 0x20,
            (1, 3) => 0x80,
            _ => 0,
        };
        
        if char_row < char_height && col < grid[0].len() {
            grid[char_row][col] |= bit;
        }
    }

    print!("\nWaveform:\n");
    for row in 0..char_height {
        for &code in &grid[row] {
            if code == 0 {
                print!(" ");
            } else {
                let c = std::char::from_u32(0x2800 + code as u32).unwrap_or(' ');
                print!("{}", c);
            }
        }
        print!("                                                                                \n");
    }
    
    let _ = io::stdout().flush();
}
