# Premonition Web UI

A Prophet-style polyphonic analog synthesizer UI, backed by the `premonition-wasm` WebAssembly engine.

## Quick Start (UI Only)

The UI works immediately without building WASM — all controls are interactive and an animated oscilloscope plays. Just open the page:

```bash
# From the workspace root, serve the web directory:
cd premonition-web
python3 -m http.server 8080
# Then open http://localhost:8080 in Chrome or Firefox
```

> **Note:** Opening `index.html` directly as `file://` works for the UI, but WASM loading and `fetch()` preset loading require a local HTTP server.

## Full Audio (WASM Build)

To enable real sound, build the WebAssembly engine first:

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build the WASM module (from workspace root)
cd premonition-wasm
wasm-pack build --target web

# Now serve and open — audio will activate when you click ACTIVATE
cd ../premonition-web
python3 -m http.server 8080
```

## Controls

| Control | Action |
|---|---|
| **Drag knob** (up/down) | Adjust parameter |
| **Shift + Drag** | Fine adjustment (5× slower) |
| **Scroll wheel** | Adjust parameter |
| **Double-click knob** | Reset to default |
| **A W S E D F T G Y H U J K** | Play notes (piano layout, C4–C5) |
| **−/+ buttons** | Shift octave |
| **Escape** | All notes off (panic) |
| **ACTIVATE button** | Start audio engine |
| **MIDI controller** | Full MIDI input (Web MIDI API) |

## MIDI CC Mapping

| CC # | Parameter |
|------|-----------|
| 74 | Filter Cutoff |
| 71 | Filter Resonance |
| 73 | Amp Attack |
| 72 | Amp Release |
| 1 (Mod Wheel) | LFO Depth |
| 7 | Master Volume |
| 10 | Osc A/B Mix |

## Preset Files

Presets are JSON files in `presets/`. The format matches the Rust `Parameters` struct (internal values, not normalized). Use **SAVE** to download the current state as a JSON file.

## File Structure

```
premonition-web/
├── index.html      # Full synth panel HTML
├── style.css       # Hardware-inspired dark panel styling
├── synth.js        # Engine, knobs, keyboard, MIDI, presets
└── presets/
    ├── init.json   # Default: clean init patch
    ├── bass.json   # Deep analog bass
    └── pad.json    # Lush unison pad
```
