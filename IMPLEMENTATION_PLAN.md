# Prophet-Style Synthesizer Implementation Plan

## Overview
A polyphonic analog-modeled subtractive synthesizer with authentic imperfections, per-voice variations, and realistic signal path modeling.

**Target Configuration**: 5 or 8 voices, Curtis/SSM-style 4-pole LPF, dual VCOs + noise

**Deployment Targets**: VST plugin, Standalone app, WebAssembly (Web App)

---

## Cross-Platform Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    HOST / UI LAYER                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   JUCE      │  │   WASM/JS   │  │  Standalone Binary  │  │
│  │  (VST/AU)   │  │  (Web App)  │  │    (Native CLI)     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
└─────────┼────────────────┼───────────────────┼──────────────┘
          │                │                   │
          ▼                ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                    HOST ADAPTER LAYER                       │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  - Audio I/O abstraction                                 ││
│  │  - MIDI input abstraction                                ││
│  │  - Parameter binding (get/set with notifications)       ││
│  │  - Thread management                                     ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
          │                │                   │
          ▼                ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                    CORE DSP ENGINE (Language-Agnostic)      │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  - All signal processing (oscillators, filter, etc.)   ││
│  │  - Voice management                                      ││
│  │  - Parameter processing                                  ││
│  │  - Effects processing                                    ││
│  │  - NO platform-specific code                             ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Language Strategy

| Target | Language | Toolchain |
|--------|----------|-----------|
| Core DSP | Rust | `rustc` + `wasm-pack` |
| VST/AU | C++ (JUCE) | Projucer → Xcode/VS |
| Web App | Rust → WASM | `wasm-pack` + JS framework |
| Standalone | Rust | `cargo` (native binary) |

### Build Outputs

| Target | Output | Distribution |
|--------|--------|--------------|
| VST2 | `.vst` (macOS/Windows/Linux) | Plugin folder |
| VST3 | `.vst3` | Plugin folder |
| AU | `.component` (macOS only) | `/Library/Audio/Plug-Ins/` |
| Web | `.wasm` + JS glue | CDN / downloadable |
| Standalone | Native binary | Direct execution |

---

## Phase 0: Project Scaffold & Architecture

### 0.1 Monorepo Structure
```
premonition/
├── Cargo.toml              (Rust workspace - core + wasm)
├── premonition-core/       (DSP engine - pure Rust, no std::os)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── audio/
│       ├── control/
│       └── effects/
├── premonition-wasm/       (WebAssembly bindings)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── premonition-juce/       (JUCE wrapper for VST/AU)
│   ├── premonition-juce.jucer
│   ├── Source/
│   │   ├── PluginProcessor.h/cpp
│   │   ├── PluginEditor.h/cpp
│   │   └── JuceWrapper.h/cpp    (binds to Rust core)
│   └── Builds/
└── premonition-cli/        (Standalone CLI app)
    ├── Cargo.toml
    └── src/
        └── main.rs
```

### 0.2 Core API Design
The core DSP engine exposes a stable C-compatible FFI interface:

```rust
// Exposed functions (exported via C ABI or wasm-bindgen)
#[no_mangle]
pub extern "C" fn premonition_create() -> *mut PremonitionEngine;

#[no_mangle]
pub extern "C" fn premonition_process(
    engine: *mut PremonitionEngine,
    output_l: *mut f32,
    output_r: *mut f32,
    num_samples: usize,
    sample_rate: f32,
);

#[no_mangle]
pub extern "C" fn premonition_note_on(
    engine: *mut PremonitionEngine,
    channel: u8,
    note: u8,
    velocity: u8,
);

#[no_mangle]
pub extern "C" fn premonition_note_off(
    engine: *mut PremonitionEngine,
    channel: u8,
    note: u8,
);

#[no_mangle]
pub extern "C" fn premonition_set_param(
    engine: *mut PremonitionEngine,
    param_id: u32,
    value: f32,
);

#[no_mangle]
pub extern "C" fn premonition_get_param(
    engine: *mut PremonitionEngine,
    param_id: u32,
) -> f32;

#[no_mangle]
pub extern "C" fn premonition_destroy(engine: *mut PremonitionEngine);
```

### 0.3 Host Adapter Requirements
Each host (JUCE, WASM, CLI) must implement:
- Audio thread callback (calls `premonition_process`)
- MIDI event handling (calls `premonition_note_on/off`)
- Parameter change propagation
- Sample rate initialization
- Reset/panic handling

---

## Phase 1: Core DSP Infrastructure

### 1.1 Project Structure & Build System
- [x] Initialize `premonition-core` Rust crate (`no_std` compatible)
- [x] Configure `premonition-wasm` for WebAssembly target
- [x] Configure `premonition-juce` JUCE project
- [x] Configure `premonition-cli` for native builds
- [x] Set up audio buffer processing pipeline
- [x] Define sample rate configuration (44.1kHz, 48kHz, 96kHz support)
- [x] Implement voice manager with round-robin allocation
- [x] Create thread-safe parameter system (atomic for cross-thread communication)
- [x] Define FFI exports for host integration

### 1.2 Voice Architecture
- [x] Define `Voice` struct with all per-voice state
- [x] Implement per-voice random seed for imperfection system
- [x] Create voice allocation/disallocation logic
- [x] Handle voice stealing with click detection
- [x] Implement unison mode (voice stacking)

### 1.3 Per-Voice Imperfection System
- [x] `Slop` parameter (0.0-1.0) controlling imperfection intensity
- [x] Per-voice oscillator drift parameters (slow random process)
- [x] Per-voice filter cutoff offset (component tolerance)
- [x] Per-voice envelope timing variance
- [x] Per-voice mixer gain variance
- [x] Per-voice DC offset

---

## Phase 2: Oscillator Section

### 2.1 VCO1 Implementation
- [x] Waveform generation: Saw, Pulse
- [x] Frequency with slight quantization/rounding
- [x] Fine tune (±7 semitones with imperfect scaling)
- [x] Pulse width with nonlinear response near extremes
- [x] Hard sync to VCO2 (non-sample-accurate reset)
- [x] LFO → pitch modulation (with delay/lag)
- [x] Envelope → pitch modulation (limited depth)

### 2.2 VCO2 Implementation
- [x] Waveform generation: Saw, Pulse, Triangle (imperfect)
- [x] Coarse tune (semitone steps with detune error)
- [x] Fine tune (imperfect scaling)
- [x] PW with different response curve than VCO1
- [x] LFO → pitch modulation
- [x] Audio-rate modulation of VCO1 (bandwidth-limited)
- [x] Variable sync behavior per voice

### 2.3 Oscillator Drift System
- [x] Slow chaotic drift model (not purely random)
- [x] Temperature-like instability simulation
- [x] Per-voice unique drift characteristics
- [x] Slop parameter integration

---

## Phase 3: Noise & Mixer

### 3.1 Noise Generator
- [x] White noise with pink tilt
- [x] Level variance between voices
- [x] Amplitude instability modeling

### 3.2 Mixer Section
- [x] Three input channels: Osc1, Osc2, Noise
- [x] Per-channel gain with soft saturation (~80%+)
- [x] Nonlinear summing (harmonic distortion)
- [x] Intermodulation artifacts
- [x] Per-voice gain variance

---

## Phase 4: Filter Section

### 4.1 4-Pole LPF Implementation
- [x] Curtis or SSM filter topology selection
- [x] 24 dB/octave rolloff
- [x] Cutoff frequency with per-voice variance
- [x] Resonance with nonlinear peak shifting
- [x] Self-oscillation (amplitude instability)
- [x] Approximate keyboard tracking

### 4.2 Filter Interactions
- [x] Envelope amount interacting with input level
- [x] Resonance behavior at high input levels
- [x] Filter modulation depth nonlinearity

---

## Phase 5: Amplifier (VCA)

### 5.1 OTA-Style VCA
- [x] Level-dependent distortion modeling
- [x] Envelope-to-VCA response (imperfect linearity)
- [x] Velocity sensitivity (optional, bypassable)

---

## Phase 6: Modulation System

### 6.1 Envelopes (2x ADSR per voice)
- [x] Attack (minimum ~2-4ms, exponential curve)
- [x] Decay with per-voice timing variance
- [x] Sustain with per-voice level differences
- [x] Release with exponential curves
- [x] Retrigger timing jitter
- [x] Output: Filter Envelope, Amp Envelope

### 6.2 Global LFO
- [x] Waveforms: Triangle, Saw, Square, Random (S&H)
- [x] Rate with internal drift
- [x] Nonlinear depth across range
- [x] Destinations: Osc pitch, PW, filter cutoff

### 6.3 Poly-Mod Section
- [x] Sources: Osc2 (audio-rate), Filter Envelope
- [x] Destinations: Osc1 freq, Osc1 PW, Filter cutoff
- [x] Asymmetrical modulation depth
- [x] Bandwidth-limited audio-rate modulation
- [x] Scaling errors

---

## Phase 7: Output Stage

### 7.1 Voice Summing
- [x] Mix all active voices
- [x] Soft-clipping stage
- [x] Output noise modeling
- [x] Asymmetry correction

### 7.2 Stereo Enhancement (Optional)
- [x] Per-voice stereo spread
- [x] Bypass option for authentic mode

---

## Phase 8: MIDI & Control

### 8.1 MIDI Input
- [x] Note On/Off handling
- [x] Velocity → Amp (optional)
- [x] Pitch bend with range
- [x] Mod wheel → LFO depth
- [x] Sustain pedal

### 8.2 Modern Enhancements (Bypassable)
- [x] Velocity → Filter/AMP
- [x] Aftertouch → Pitch/Filter
- [x] MPE support

### 8.3 Parameter System
- [x] All synth parameters exposed
- [x] Smoothing/interpolation for parameter changes
- [ ] MIDI CC mapping

---

## Phase 9: Effects (Optional)

- [x] Chorus (authentic BBD-style)
- [x] Delay
- [x] Reverb
- [x] Global bypass for authentic mode

---

## Phase 10: Host Adapters

### 10.1 JUCE Wrapper (VST/AU)
- [x] Create JUCE project via Projucer
- [x] Implement `AudioProcessor` subclass
- [ ] Bind to Rust core via FFI/Cargo build
- [x] Handle audio processing callbacks
- [x] Route MIDI messages to engine
- [x] Implement parameter state (JuceVST or VST3)
- [ ] Basic editor (or bridge to web UI)
- [ ] Build for VST2, VST3, AU targets

### 10.2 WebAssembly Build
- [x] Configure `wasm-pack` build
- [x] Export engine functions via `wasm-bindgen`
- [x] Create JavaScript wrapper class
- [ ] Web Audio API integration
- [ ] Parameter binding helpers
- [ ] Build and test in browser
- [ ] Performance optimization (wasm-opt)

### 10.3 Standalone CLI
- [x] Implement audio I/O with `cpal` or `rodio`
- [x] Add MIDI input support (`midir`)
- [x] Computer keyboard input with piano layout (A=C4)
- [ ] Optional: Add TUI with `ratatui` or `cursive`
- [ ] Optional: Headless mode for server/audio rendering
- [ ] Package for macOS, Windows, Linux

### 10.4 Web UI (Optional)
- [x] Create JS/TS web application
- [x] Canvas-based synthesizer UI or HTML controls
- [x] Load WASM module
- [x] Real-time parameter updates
- [x] Preset management (localStorage or server)
- [ ] Deploy to CDN or static hosting

---

## Phase 11: Testing & Calibration

### 11.1 Unit Testing (Core)
- [ ] Oscillator waveform generation tests
- [ ] Filter frequency response tests
- [ ] Envelope timing tests
- [ ] Modulation routing tests
- [ ] Parameter serialization tests

### 11.2 Cross-Platform Validation
- [ ] Test VST in DAW (Ableton, Logic, etc.)
- [ ] Test AU in Logic/macOS
- [ ] Test WASM in Chrome, Firefox, Safari
- [ ] Test CLI on macOS, Windows, Linux

### 11.3 Audio Testing (All Targets)
- [ ] Frequency analysis (verify harmonic content)
- [ ] Transient response verification
- [ ] Per-voice variation audibility
- [ ] CPU usage profiling per platform
- [ ] Buffer size latency testing

### 11.4 Performance Optimization
- [ ] SIMD optimization for filter (std::simd or portable_simd)
- [ ] WASM binary size reduction
- [ ] JUCE plugin scanning validation

---

## Implementation Order & Dependencies

```
Phase 0: Project Scaffold (establish cross-platform structure)
    ↓
Phase 1: Core DSP Infrastructure (engine, voices, parameters)
    ↓
Phase 2: VCO1 ←→ VCO2 (can implement in parallel)
    ↓
Phase 3: Noise → Mixer
    ↓
Phase 4: Filter ←─── Mixer output
    ↓
Phase 5: VCA ←────── Filter output
    ↓
Phase 6: Modulation System (envelopes, LFO, poly-mod)
    ↓
Phase 7: Output Stage
    ↓
Phase 8: MIDI & Control
    ↓
Phase 9: Effects (parallel with Phase 8)
    ↓
Phase 10: Host Adapters (JUCE, WASM, CLI)
    ↓
Phase 11: Testing & Calibration (all targets)
```

### Parallel Workstreams
```
Phase 0 ──────────────────────────────────────────────────────
    │
    ├─► Phase 1 (Core DSP)
    │       │
    │       ├─► Phase 2 (Oscillators)
    │       │       │
    │       │       ├─► Phase 3 (Noise/Mixer)
    │       │       │       │
    │       │       │       └─► Phase 4 (Filter)
    │       │       │               │
    │       │       │               └─► Phase 5 (VCA)
    │       │       │
    │       │       └─► Phase 6 (Modulation)
    │       │
    │       └─► Phase 7 (Output)
    │
    └─► Phase 10 (Host Adapters) ──── runs in parallel after Phase 7
            │
            ├─► JUCE wrapper
            ├─► WASM bindings
            └─► CLI app
```

---

## Parameter List

| Parameter | Range | Default |
|-----------|-------|---------|
| Osc1 Wave | Saw/Pulse | Saw |
| Osc1 Freq | 20Hz-20kHz | 440Hz |
| Osc1 Fine | ±7 semitones | 0 |
| Osc1 PW | 0-100% | 50% |
| Osc2 Wave | Saw/Pulse/Tri | Saw |
| Osc2 Coarse | ±24 semitones | 0 |
| Osc2 Fine | ±7 semitones | 0 |
| Osc2 PW | 0-100% | 50% |
| Noise Level | 0-100% | 0% |
| Osc Mix | 0-100% | 50% |
| Filter Cutoff | 20Hz-20kHz | 5kHz |
| Filter Resonance | 0-100% | 0% |
| Filter Env Amount | -100% to +100% | 0% |
| Amp Attack | 0-10s | 1ms |
| Amp Decay | 0-10s | 200ms |
| Amp Sustain | 0-100% | 50% |
| Amp Release | 0-10s | 300ms |
| Filter Attack | 0-10s | 1ms |
| Filter Decay | 0-10s | 200ms |
| Filter Sustain | 0-100% | 50% |
| Filter Release | 0-10s | 300ms |
| LFO Rate | 0.01-100Hz | 1Hz |
| LFO Wave | Tri/Saw/Sq/Random | Triangle |
| LFO→Osc | 0-100% | 0% |
| LFO→Filter | 0-100% | 0% |
| LFO→PW | 0-100% | 0% |
| PolyMod→Osc1Freq | -100% to +100% | 0% |
| PolyMod→Osc1PW | -100% to +100% | 0% |
| PolyMod→Filter | -100% to +100% | 0% |
| Slop | 0-100% | 10% |
| Unison Voices | 1-8 | 1 |
| Unison Detune | 0-100% | 0% |
| Master Volume | 0-100% | 75% |

---

## File Structure (Full Monorepo)

```
premonition/
├── Cargo.toml                    (workspace manifest)
│
├── premonition-core/             # Pure Rust DSP engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs               (module root, FFI exports)
│   │   ├── prelude.rs           (re-exported types)
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs        (main audio processing loop)
│   │   │   ├── voice.rs         (voice state)
│   │   │   ├── voice_manager.rs (allocation, stealing)
│   │   │   ├── oscillator.rs    (VCO1, VCO2)
│   │   │   ├── noise.rs         (noise generator)
│   │   │   ├── mixer.rs         (3-input mixer)
│   │   │   ├── filter.rs        (4-pole Curtis/SSM LPF)
│   │   │   ├── vca.rs           (OTA-style amplifier)
│   │   │   ├── envelope.rs      (ADSR)
│   │   │   ├── lfo.rs           (global LFO)
│   │   │   ├── poly_mod.rs      (poly-mod routing)
│   │   │   ├── output.rs        (voice summing, soft clip)
│   │   │   └── imperfection.rs  (slop, drift system)
│   │   ├── control/
│   │   │   ├── mod.rs
│   │   │   ├── params.rs        (parameter definitions, IDs)
│   │   │   ├── smoothing.rs     (parameter interpolation)
│   │   │   └── midi.rs          (MIDI event types)
│   │   └── effects/
│   │       ├── mod.rs
│   │       ├── chorus.rs
│   │       ├── delay.rs
│   │       └── reverb.rs
│   └── tests/                   (core DSP unit tests)
│
├── premonition-wasm/             # WebAssembly bindings
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs               (wasm-bindgen exports)
│   │   └── js/
│   │       └── audio_context.rs (Web Audio API glue)
│   └── pkg/                     (wasm-pack output)
│
├── premonition-juce/             # JUCE wrapper (VST/AU)
│   ├── premonition-juce.jucer   (Projucer project)
│   ├── Source/
│   │   ├── PluginProcessor.h
│   │   ├── PluginProcessor.cpp  (audio thread, FFI calls)
│   │   ├── PluginEditor.h
│   │   ├── PluginEditor.cpp    (GUI - or use JS/web UI)
│   │   └── PremonitionWrapper.h/cpp
│   ├── Builds/
│   │   ├── MacOSX/
│   │   ├── Windows/
│   │   └── LinuxMakefile/
│   └── Resources/
│       └── Icon.icns
│
└── premonition-cli/              # Standalone CLI app
    ├── Cargo.toml
    ├── src/
    │   └── main.rs              (cpal + rodio or similar)
    └── assets/
        └── presets/
```

---

## Host Integration Guide

### JUCE Integration (VST/AU/Standalone)
```cpp
// In PluginProcessor.cpp
#include "PremonitionWrapper.h"

class PremonitionProcessor : public AudioProcessor {
    premonition::Engine engine;
    
    void prepareToPlay(double sampleRate, ...) override {
        engine.init(sampleRate);
    }
    
    void processBlock(AudioBuffer& buffer, MidiBuffer& midi) override {
        // Convert MIDI → engine calls
        for (auto event : midi) {
            auto msg = event.getMessage();
            if (msg.isNoteOn()) {
                engine.note_on(msg.getChannel(), msg.getNoteNumber(), msg.getVelocity());
            } else if (msg.isNoteOff()) {
                engine.note_off(msg.getChannel(), msg.getNoteNumber());
            }
        }
        
        // Process audio
        engine.process(
            buffer.getWritePointer(0),
            buffer.getWritePointer(1),
            buffer.getNumSamples(),
            (float)getSampleRate()
        );
    }
};
```

### WebAssembly Integration
```javascript
// Web app usage
import init, { Engine } from './pkg/premonition_wasm.js';

async function run() {
    await init();
    const engine = new Engine();
    engine.init(48000);
    
    // Connect to Web Audio API
    const ctx = new AudioContext();
    const node = ctx.createScriptProcessor(4096, 0, 2);
    node.onaudioprocess = (e) => {
        engine.process(
            e.outputBuffer.getChannelData(0),
            e.outputBuffer.getChannelData(1),
            4096
        );
    };
    
    // UI binds to engine.set_param() / engine.get_param()
}
```

### CLI Integration
```rust
// premonition-cli/src/main.rs
use premonition_core::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.init(48000.0);
    
    // Use cpal for audio I/O
    // Use rtmidi for MIDI input
    // Simple TUI or no UI (headless mode)
}
```

---

## Build Configuration

### Cargo Workspace (Root)
```toml
# Cargo.toml
[workspace]
members = [
    "premonition-core",
    "premonition-wasm",
    "premonition-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name"]
license = "MIT"

[workspace.dependencies]
# Core dependencies shared across crates
rust-version = "1.70"
```

### Core DSP Crate
```toml
# premonition-core/Cargo.toml
[package]
name = "premonition-core"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]  # WASM + library

[features]
default = []
std = ["dep:fixed", "dep:heapless"]  # Enable std in CLI only

[dependencies]
num-traits = "0.2"
num-complex = "0.4"
```

### WASM Crate
```toml
# premonition-wasm/Cargo.toml
[package]
name = "premonition-wasm"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
premonition-core = { path = "../premonition-core", default-features = false }

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### CLI Crate
```toml
# premonition-cli/Cargo.toml
[package]
name = "premonition-cli"
version.workspace = true
edition.workspace = true

[dependencies]
premonition-core = { path = "../premonition-core", features = ["std"] }
cpal = "0.15"
midir = "0.10"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Build Commands
```bash
# Build WASM
wasm-pack build --target web --out-dir ./dist

# Build CLI
cargo build --release --manifest-path premonition-cli/Cargo.toml

# Build JUCE (via Projucer or CMake)
# Open premonition-juce/premonition-juce.jucer in Projucer
# Or use cmake: cd premonition-juce && mkdir build && cd build && cmake ..
```

---

## Notes

1. **Floating Point Precision**: Use `f64` for internal calculations, dither to `f32` output
2. **Oversampling**: Consider 2x-4x oversampling for oscillators and filter
3. **CPU Optimization**: Pre-compute wavetables, consider SIMD with `std::simd`
4. **Memory**: Pre-allocate voice and buffer storage, avoid allocations in audio thread
5. **Preset System**: Serialize `Params` struct to JSON (works across all targets)
6. **Thread Safety**: Use `Arc<AtomicF32>` for parameters, lock-free queues for MIDI
7. **WASM Considerations**: Disable dynamic dispatch, precompute sine tables, avoid `std::collections`
8. **JUCE Considerations**: Use `AudioProcessorValueTreeState` for parameter binding
