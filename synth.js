'use strict';

/* ============================================================
   PREMONITION SYNTHESIZER  —  synth.js
   Engine wrapper, audio init, knob controllers, keyboard,
   preset management, oscilloscope, and Web MIDI.
   ============================================================ */

// ── PARAMETER IDs ────────────────────────────────────────────
// Must match the Rust ParameterId enum (0–35)
const P = Object.freeze({
  OSC1_WAVE:  0,  OSC1_FREQ:   1,  OSC1_FINE:  2,  OSC1_PW:      3,  OSC1_PENV: 4,
  OSC2_WAVE:  5,  OSC2_COARSE: 6,  OSC2_FINE:  7,  OSC2_PW:      8,
  NOISE:      9,  OSC_MIX:    10,
  FILT_CUT:  11,  FILT_RES:   12,  FILT_ENV:  13,  FILT_KEY:    14,
  AMP_ATK:   15,  AMP_DEC:    16,  AMP_SUS:   17,  AMP_REL:     18,
  FLT_ATK:   19,  FLT_DEC:    20,  FLT_SUS:   21,  FLT_REL:     22,
  LFO_RATE:  23,  LFO_WAVE:   24,  LFO_DEPTH: 25,
  LFO_OSC:   26,  LFO_FILT:   27,  LFO_PW:    28,
  PM_FREQ:   29,  PM_PW:      30,  PM_FILT:   31,
  SLOP:      32,  UNISON:     33,  UNISON_DET:34,
  MASTER_VOL:35,
});

// Default normalized (0–1) values for every parameter
const DEFAULTS = {
  [P.OSC1_WAVE]:  0,    [P.OSC1_FREQ]:   0.5,  [P.OSC1_FINE]:  0.5,
  [P.OSC1_PW]:    0.5,  [P.OSC1_PENV]:   0.5,
  [P.OSC2_WAVE]:  0,    [P.OSC2_COARSE]: 0.5,  [P.OSC2_FINE]:  0.5,  [P.OSC2_PW]: 0.5,
  [P.NOISE]:      0,    [P.OSC_MIX]:     0.5,
  [P.FILT_CUT]:   0.25, [P.FILT_RES]:    0,    [P.FILT_ENV]:   0.5,  [P.FILT_KEY]: 0,
  [P.AMP_ATK]:    0.03, [P.AMP_DEC]:     0.3,  [P.AMP_SUS]:    0.7,  [P.AMP_REL]: 0.3,
  [P.FLT_ATK]:    0.03, [P.FLT_DEC]:     0.3,  [P.FLT_SUS]:    0.5,  [P.FLT_REL]: 0.3,
  [P.LFO_RATE]:   0.1,  [P.LFO_WAVE]:    0,    [P.LFO_DEPTH]:  0.5,
  [P.LFO_OSC]:    0.5,  [P.LFO_FILT]:    0,    [P.LFO_PW]:     0.5,
  [P.PM_FREQ]:    0.5,  [P.PM_PW]:       0.5,  [P.PM_FILT]:    0.5,
  [P.SLOP]:       0.1,  [P.UNISON]:      0,    [P.UNISON_DET]: 0,
  [P.MASTER_VOL]: 0.75,
};

// Inline init preset (raw internal values from Rust Parameters struct)
// Used as fallback when fetch() is unavailable (e.g., file:// protocol)
const BUILTIN_PRESETS = {
  init: {
    osc1_wave:0, osc1_freq:0.0, osc1_fine:0.0, osc1_pulse_width:0.5, osc1_pitch_env_depth:0.0,
    osc2_wave:0, osc2_coarse:0.0, osc2_fine:0.0, osc2_pulse_width:0.5,
    noise_level:0.0, osc_mix:0.5,
    filter_cutoff:20000.0, filter_resonance:0.0, filter_env_amount:0.0, filter_keyboard_tracking:0.0,
    amp_attack:0.01, amp_decay:0.2, amp_sustain:1.0, amp_release:0.3,
    filter_attack:0.01, filter_decay:0.2, filter_sustain:1.0, filter_release:0.3,
    lfo_rate:1.0, lfo_wave:0, lfo_depth:0.5,
    lfo_to_osc:0.0, lfo_to_filter:0.0, lfo_to_pw:0.0,
    poly_mod_osc2_to_osc1_freq:0.0, poly_mod_osc2_to_osc1_pw:0.0, poly_mod_filter_env_to_filter:0.0,
    slop:0.1, unison_voices:1, unison_detune:0.0, master_volume:0.75,
  },
  bass: {
    osc1_wave:1, osc1_freq:0.0, osc1_fine:0.0, osc1_pulse_width:0.5, osc1_pitch_env_depth:0.0,
    osc2_wave:0, osc2_coarse:-12.0, osc2_fine:0.05, osc2_pulse_width:0.5,
    noise_level:0.05, osc_mix:0.7,
    filter_cutoff:400.0, filter_resonance:0.3, filter_env_amount:0.8, filter_keyboard_tracking:0.5,
    amp_attack:0.002, amp_decay:0.1, amp_sustain:0.4, amp_release:0.1,
    filter_attack:0.005, filter_decay:0.15, filter_sustain:0.1, filter_release:0.2,
    lfo_rate:0.5, lfo_wave:0, lfo_depth:0.2,
    lfo_to_osc:0.0, lfo_to_filter:0.0, lfo_to_pw:0.0,
    poly_mod_osc2_to_osc1_freq:0.0, poly_mod_osc2_to_osc1_pw:0.0, poly_mod_filter_env_to_filter:0.0,
    slop:0.2, unison_voices:2, unison_detune:0.1, master_volume:1.0,
  },
  pad: {
    osc1_wave:2, osc1_freq:0.0, osc1_fine:0.0, osc1_pulse_width:0.5, osc1_pitch_env_depth:0.0,
    osc2_wave:2, osc2_coarse:0.01, osc2_fine:-0.05, osc2_pulse_width:0.5,
    noise_level:0.0, osc_mix:0.5,
    filter_cutoff:1200.0, filter_resonance:0.2, filter_env_amount:0.4, filter_keyboard_tracking:0.8,
    amp_attack:1.2, amp_decay:0.5, amp_sustain:0.8, amp_release:2.5,
    filter_attack:2.0, filter_decay:1.0, filter_sustain:0.5, filter_release:3.0,
    lfo_rate:0.2, lfo_wave:0, lfo_depth:0.4,
    lfo_to_osc:0.005, lfo_to_filter:500.0, lfo_to_pw:0.0,
    poly_mod_osc2_to_osc1_freq:0.0, poly_mod_osc2_to_osc1_pw:0.0, poly_mod_filter_env_to_filter:0.0,
    slop:0.15, unison_voices:4, unison_detune:0.2, master_volume:0.8,
  },
};


// Map JSON field name → parameter ID
const FIELD_TO_PARAM = {
  osc1_wave: P.OSC1_WAVE, osc1_freq: P.OSC1_FREQ, osc1_fine: P.OSC1_FINE,
  osc1_pulse_width: P.OSC1_PW, osc1_pitch_env_depth: P.OSC1_PENV,
  osc2_wave: P.OSC2_WAVE, osc2_coarse: P.OSC2_COARSE, osc2_fine: P.OSC2_FINE,
  osc2_pulse_width: P.OSC2_PW,
  noise_level: P.NOISE, osc_mix: P.OSC_MIX,
  filter_cutoff: P.FILT_CUT, filter_resonance: P.FILT_RES,
  filter_env_amount: P.FILT_ENV, filter_keyboard_tracking: P.FILT_KEY,
  amp_attack: P.AMP_ATK, amp_decay: P.AMP_DEC,
  amp_sustain: P.AMP_SUS, amp_release: P.AMP_REL,
  filter_attack: P.FLT_ATK, filter_decay: P.FLT_DEC,
  filter_sustain: P.FLT_SUS, filter_release: P.FLT_REL,
  lfo_rate: P.LFO_RATE, lfo_wave: P.LFO_WAVE, lfo_depth: P.LFO_DEPTH,
  lfo_to_osc: P.LFO_OSC, lfo_to_filter: P.LFO_FILT, lfo_to_pw: P.LFO_PW,
  poly_mod_osc2_to_osc1_freq: P.PM_FREQ, poly_mod_osc2_to_osc1_pw: P.PM_PW,
  poly_mod_filter_env_to_filter: P.PM_FILT,
  slop: P.SLOP, unison_voices: P.UNISON, unison_detune: P.UNISON_DET,
  master_volume: P.MASTER_VOL,
};

// Convert a raw internal value (as stored in Parameters struct) to normalized 0–1
// Mirrors the inverse of Parameters::set() in Rust params.rs
function toNorm(id, v) {
  switch (id) {
    case P.OSC1_WAVE:    return Math.min(1, v / 1);
    case P.OSC1_FREQ:    return Math.max(0, Math.min(1, v / 48 + 0.5));
    case P.OSC1_FINE:    return Math.max(0, Math.min(1, v / 2 + 0.5));
    case P.OSC1_PW:      return Math.max(0, Math.min(1, v));
    case P.OSC1_PENV:    return Math.max(0, Math.min(1, v / 0.5 + 0.5));
    case P.OSC2_WAVE:    return Math.min(1, v / 2);
    case P.OSC2_COARSE:  return Math.max(0, Math.min(1, v / 48 + 0.5));
    case P.OSC2_FINE:    return Math.max(0, Math.min(1, v / 2 + 0.5));

    case P.OSC2_PW:      return Math.max(0, Math.min(1, v));
    case P.NOISE:        return Math.max(0, Math.min(1, v));
    case P.OSC_MIX:      return Math.max(0, Math.min(1, v));
    case P.FILT_CUT:     return Math.max(0, Math.min(1, (v - 20) / 19980));
    case P.FILT_RES:     return Math.max(0, Math.min(1, v));
    case P.FILT_ENV:     return Math.max(0, Math.min(1, v / 2 + 0.5));
    case P.FILT_KEY:     return Math.max(0, Math.min(1, v));
    case P.AMP_ATK:      return Math.max(0, Math.min(1, Math.cbrt(v / 10)));
    case P.AMP_DEC:      return Math.max(0, Math.min(1, Math.cbrt(v / 10)));
    case P.AMP_SUS:      return Math.max(0, Math.min(1, v));
    case P.AMP_REL:      return Math.max(0, Math.min(1, Math.cbrt(v / 10)));
    case P.FLT_ATK:      return Math.max(0, Math.min(1, Math.cbrt(v / 10)));
    case P.FLT_DEC:      return Math.max(0, Math.min(1, Math.cbrt(v / 10)));
    case P.FLT_SUS:      return Math.max(0, Math.min(1, v));
    case P.FLT_REL:      return Math.max(0, Math.min(1, Math.cbrt(v / 10)));
    case P.LFO_RATE:     return Math.max(0, Math.min(1, Math.sqrt((v - 0.01) / 100)));
    case P.LFO_WAVE:     return Math.min(1, v / 3);
    case P.LFO_DEPTH:    return Math.max(0, Math.min(1, v));
    case P.LFO_OSC:      return Math.max(0, Math.min(1, v / 0.1 + 0.5));
    case P.LFO_FILT:     return Math.max(0, Math.min(1, v / 5000));
    case P.LFO_PW:       return Math.max(0, Math.min(1, v / 0.5 + 0.5));
    case P.PM_FREQ:      return Math.max(0, Math.min(1, v / 2 + 0.5));
    case P.PM_PW:        return Math.max(0, Math.min(1, v / 2 + 0.5));
    case P.PM_FILT:      return Math.max(0, Math.min(1, v + 0.5));
    case P.SLOP:         return Math.max(0, Math.min(1, v));
    case P.UNISON:       return Math.max(0, Math.min(1, (v - 1) / 7));
    case P.UNISON_DET:   return Math.max(0, Math.min(1, v));
    case P.MASTER_VOL:   return Math.max(0, Math.min(1, v));
    default:             return Math.max(0, Math.min(1, v));
  }
}

// ═══════════════════════════════════════════════════════════
// ENGINE WRAPPER
// ═══════════════════════════════════════════════════════════
class SynthEngine {
  constructor() {
    this.wasm       = null;
    this.audioCtx   = null;
    this.processor  = null;
    this.params     = { ...DEFAULTS };
    this.bufSize    = 512;
    this.oscBuf     = new Float32Array(4096);
    this.oscWrPos   = 0;
    this.running    = false;
  }

  async start() {
    if (this.running) return;

    this.audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    if (this.audioCtx.state === 'suspended') await this.audioCtx.resume();

    // ScriptProcessorNode — widely compatible, good enough for demo
    // Upgrade path: replace with AudioWorklet + SharedArrayBuffer
    this.processor = this.audioCtx.createScriptProcessor(this.bufSize, 0, 2);
    this.processor.onaudioprocess = (e) => this._audioCallback(e);
    this.processor.connect(this.audioCtx.destination);

    this.running = true;
    await this._loadWasm();
  }

  async _loadWasm() {
    try {
      // WASM lives in ./pkg/ relative to this file.
      // Build with: cd premonition-wasm && wasm-pack build --target web --out-dir ../premonition-web/pkg
      const mod = await import('./pkg/premonition_wasm.js');
      await mod.default();
      this.wasm = new mod.WasmEngine();
      this.wasm.init(this.audioCtx.sampleRate);
      for (const [id, val] of Object.entries(this.params)) {
        this.wasm.set_param(+id, val);
      }
      document.getElementById('audio-indicator').classList.add('on');
      console.info('[Premonition] WASM engine loaded ✓ (%.0f Hz)', this.audioCtx.sampleRate);
    } catch (err) {
      console.warn('[Premonition] WASM unavailable —', err.message);
      console.info('[Premonition] UI is functional; build premonition-wasm to enable audio.');
    }
  }

  _audioCallback(e) {
    const out0 = e.outputBuffer.getChannelData(0);
    const out1 = e.outputBuffer.getChannelData(1);

    if (this.wasm) {
      const L = new Float32Array(out0.length);
      const R = new Float32Array(out1.length);
      try {
        this.wasm.process(L, R);
        out0.set(L);
        out1.set(R);
      } catch {
        out0.fill(0); out1.fill(0);
      }
      // Store left channel for oscilloscope visualisation
      for (let i = 0, n = L.length; i < n; i++) {
        this.oscBuf[this.oscWrPos] = L[i];
        this.oscWrPos = (this.oscWrPos + 1) % this.oscBuf.length;
      }
    } else {
      out0.fill(0);
      out1.fill(0);
    }
  }

  noteOn(note, velocity = 100) { if (this.wasm) this.wasm.note_on(0, note, velocity); }
  noteOff(note)                { if (this.wasm) this.wasm.note_off(0, note); }
  allNotesOff()                { if (this.wasm) this.wasm.all_notes_off(); }

  setParam(id, value) {
    this.params[id] = value;
    if (this.wasm) this.wasm.set_param(id, value);
  }

  loadParamMap(normalizedMap) {
    for (const [id, val] of Object.entries(normalizedMap)) {
      this.setParam(+id, val);
    }
  }
}

const engine = new SynthEngine();
window.premonitionEngine = engine; // expose for debugging

// ═══════════════════════════════════════════════════════════
// KNOB CONTROL
// ═══════════════════════════════════════════════════════════
const allKnobs   = new Map(); // paramId → KnobControl
const allSwitches = new Map(); // paramId → WaveSwitchControl

class KnobControl {
  constructor(el) {
    this.el          = el;
    this.paramId     = +el.dataset.param;
    this.defaultVal  = parseFloat(el.dataset.default ?? 0.5);
    this.value       = DEFAULTS[this.paramId] ?? this.defaultVal;
    this.indicatorEl = el.querySelector('.knob-indicator');
    this.valueEl     = el.closest('.control')?.querySelector('.knob-value');
    this._drag       = false;
    this._y0         = 0;
    this._v0         = 0;
    this._attach();
    this._render();
  }

  _attach() {
    // ── Mouse drag ──
    this.el.addEventListener('mousedown', (e) => {
      e.preventDefault();
      this._drag = true; this._y0 = e.clientY; this._v0 = this.value;
      this.el.classList.add('active');
      document.body.style.cursor = 'ns-resize';
    });
    document.addEventListener('mousemove', (e) => {
      if (!this._drag) return;
      const sens = e.shiftKey ? 1000 : 200;
      this._set(clamp01(this._v0 + (this._y0 - e.clientY) / sens));
    });
    document.addEventListener('mouseup', () => {
      if (this._drag) {
        this._drag = false;
        this.el.classList.remove('active');
        document.body.style.cursor = '';
      }
    });

    // ── Scroll wheel ──
    this.el.addEventListener('wheel', (e) => {
      e.preventDefault();
      const step = e.shiftKey ? 0.001 : 0.01;
      this._set(clamp01(this.value - Math.sign(e.deltaY) * step));
    }, { passive: false });

    // ── Double-click reset ──
    this.el.addEventListener('dblclick', () => this._set(this.defaultVal));

    // ── Touch ──
    let ty0 = 0, tv0 = 0;
    this.el.addEventListener('touchstart', (e) => { ty0 = e.touches[0].clientY; tv0 = this.value; }, { passive: true });
    this.el.addEventListener('touchmove',  (e) => {
      e.preventDefault();
      this._set(clamp01(tv0 + (ty0 - e.touches[0].clientY) / 200));
    }, { passive: false });
  }

  _set(v) { this.value = v; this._render(); engine.setParam(this.paramId, v); }

  /** External update (from preset load / MIDI CC) — does NOT re-send to engine */
  setValue(v) { this.value = v; this._render(); }

  _render() {
    // 0→1 mapped to –135°→+135° (270° sweep)
    const angle = -135 + this.value * 270;
    if (this.indicatorEl) this.indicatorEl.style.transform = `rotate(${angle}deg)`;
    if (this.valueEl)     this.valueEl.textContent = this.value.toFixed(2);
  }
}

// ═══════════════════════════════════════════════════════════
// WAVE SWITCH CONTROL
// ═══════════════════════════════════════════════════════════
class WaveSwitchControl {
  constructor(el) {
    this.el      = el;
    this.paramId = +el.dataset.param;
    this.count   = +el.dataset.count;
    this.index   = 0;
    this.buttons = [...el.querySelectorAll('.wave-btn')];
    this.buttons.forEach((btn, i) => btn.addEventListener('click', () => this._set(i)));
  }

  _set(i) {
    this.index = i;
    this.buttons.forEach((b, j) => b.classList.toggle('active', j === i));
    const norm = this.count <= 1 ? 0 : i / (this.count - 1);
    engine.setParam(this.paramId, norm);
  }

  /** External update — does NOT re-send to engine */
  setIndex(i) {
    this.index = Math.max(0, Math.min(this.count - 1, i));
    this.buttons.forEach((b, j) => b.classList.toggle('active', j === this.index));
  }
}

function initControls() {
  document.querySelectorAll('.knob[data-param]').forEach(el => {
    const k = new KnobControl(el);
    allKnobs.set(k.paramId, k);
  });
  document.querySelectorAll('.wave-switch[data-param]').forEach(el => {
    const s = new WaveSwitchControl(el);
    allSwitches.set(s.paramId, s);
  });
}

// ═══════════════════════════════════════════════════════════
// PRESET MANAGEMENT
// ═══════════════════════════════════════════════════════════
function applyPresetData(rawJson) {
  const normMap = {};
  for (const [field, internal] of Object.entries(rawJson)) {
    const id = FIELD_TO_PARAM[field];
    if (id === undefined) continue;
    const n = toNorm(id, internal);
    normMap[id] = n;

    if (allKnobs.has(id))    allKnobs.get(id).setValue(n);
    if (allSwitches.has(id)) {
      const sw = allSwitches.get(id);
      sw.setIndex(Math.round(internal));
    }
  }
  engine.loadParamMap(normMap);
}

async function loadPreset(name) {
  // 1. Try fetching from presets/ (works when served over HTTP)
  try {
    const resp = await fetch(`presets/${name}.json?v=${Date.now()}`);
    if (resp.ok) {
      const data = await resp.json();
      applyPresetData(data);
      console.info(`[Premonition] Loaded preset '${name}' from file`);
      return;
    }
  } catch (_) { /* fall through */ }

  // 2. Built-in fallback (works with file:// protocol)
  if (BUILTIN_PRESETS[name]) {
    applyPresetData(BUILTIN_PRESETS[name]);
    console.info(`[Premonition] Loaded built-in preset '${name}'`);
  } else {
    console.warn(`[Premonition] Preset '${name}' not found`);
  }
}

function savePresetFile() {
  // Serialize the current normalized param state for download
  const json = JSON.stringify(engine.params, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url  = URL.createObjectURL(blob);
  const a    = Object.assign(document.createElement('a'), { href: url, download: 'premonition-preset.json' });
  a.click();
  URL.revokeObjectURL(url);
}

function initPresetBar() {
  const sel   = document.getElementById('preset-select');
  const prev  = document.getElementById('prev-preset');
  const next  = document.getElementById('next-preset');
  const save  = document.getElementById('save-preset');
  const keys  = Object.keys(BUILTIN_PRESETS);

  sel.addEventListener('change', () => loadPreset(sel.value));

  prev.addEventListener('click', () => {
    const i = keys.indexOf(sel.value);
    sel.value = keys[(i - 1 + keys.length) % keys.length];
    loadPreset(sel.value);
  });
  next.addEventListener('click', () => {
    const i = keys.indexOf(sel.value);
    sel.value = keys[(i + 1) % keys.length];
    loadPreset(sel.value);
  });
  save.addEventListener('click', savePresetFile);
}

// ═══════════════════════════════════════════════════════════
// PIANO KEYBOARD
// ═══════════════════════════════════════════════════════════
const KEY_TO_SEMITONE = {
  'a':0,'w':1,'s':2,'e':3,'d':4,'f':5,'t':6,'g':7,'y':8,'h':9,'u':10,'j':11,'k':12,
};
const IS_BLACK = n => [1,3,6,8,10].includes(n % 12);

// Key letter hints mapped to MIDI note relative to C4 (60)
const KEY_HINT_FOR_OFFSET = {0:'A',1:'W',2:'S',3:'E',4:'D',5:'F',6:'T',7:'G',8:'Y',9:'H',10:'U',11:'J',12:'K'};

let octaveOffset  = 0; // ±semitones (multiples of 12)
const activeNotes = new Set();

function buildKeyboard() {
  const container = document.getElementById('keyboard');
  container.innerHTML = '';

  const baseNote = 60 + octaveOffset; // Middle C + octave shift
  const totalKeys = 15; // ~2 octaves rendered

  const whitePosX = []; // x-offset of each rendered white key
  let wx = 0;

  for (let offset = 0; offset < totalKeys; offset++) {
    const note = baseNote + offset;
    if (!IS_BLACK(note)) {
      const key = document.createElement('div');
      key.className = 'piano-key white';
      key.dataset.note = note;
      key.style.left = `${wx}px`;

      // Keyboard letter hint (only for first 13 keys)
      const hint = KEY_HINT_FOR_OFFSET[offset];
      if (hint) {
        const lbl = document.createElement('span');
        lbl.className = 'key-lbl';
        lbl.textContent = hint;
        key.appendChild(lbl);
      }

      key.addEventListener('mousedown', e => { e.preventDefault(); triggerNote(note); });
      key.addEventListener('mouseup',   () => releaseNote(note));
      key.addEventListener('mouseleave',() => releaseNote(note));
      container.appendChild(key);
      whitePosX[offset] = wx;
      wx += 29; // white key width + 1px gap
    }
  }

  // Black keys — positioned relative to their left white-key neighbour
  for (let offset = 1; offset < totalKeys; offset++) {
    const note = baseNote + offset;
    if (!IS_BLACK(note)) continue;

    const leftWhiteOffset = offset - 1;
    const leftX = whitePosX[leftWhiteOffset];
    if (leftX === undefined) continue;

    const key = document.createElement('div');
    key.className = 'piano-key black';
    key.dataset.note = note;
    // Center the 18px black key between the two flanking white keys (29px wide)
    key.style.left = `${leftX + 20}px`; // centre 18px black key over 29px white-key gap

    // Keyboard letter hint
    const hint = KEY_HINT_FOR_OFFSET[offset];
    if (hint) {
      const lbl = document.createElement('span');
      lbl.className = 'key-lbl';
      lbl.textContent = hint;
      key.appendChild(lbl);
    }

    key.addEventListener('mousedown', e => { e.preventDefault(); triggerNote(note); });
    key.addEventListener('mouseup',   () => releaseNote(note));
    key.addEventListener('mouseleave',() => releaseNote(note));
    container.appendChild(key);
  }

  container.style.width = `${wx}px`;
}

function triggerNote(note) {
  if (activeNotes.has(note)) return;
  activeNotes.add(note);
  engine.noteOn(note, 100);
  const el = document.querySelector(`.piano-key[data-note="${note}"]`);
  if (el) el.classList.add('pressed');
}

function releaseNote(note) {
  if (!activeNotes.has(note)) return;
  activeNotes.delete(note);
  engine.noteOff(note);
  const el = document.querySelector(`.piano-key[data-note="${note}"]`);
  if (el) el.classList.remove('pressed');
}

function initKeyboardInput() {
  const held = new Set();

  document.addEventListener('keydown', e => {
    if (e.repeat) return;
    if (['INPUT','SELECT','TEXTAREA'].includes(e.target.tagName)) return;

    const keyLower = e.key.toLowerCase();

    // Octave controls (Z/X)
    if (keyLower === 'z') {
      document.getElementById('oct-down').click();
      return;
    }
    if (keyLower === 'x') {
      document.getElementById('oct-up').click();
      return;
    }

    const offset = KEY_TO_SEMITONE[keyLower];
    if (offset !== undefined && !held.has(e.key)) {
      held.add(e.key);
      triggerNote(60 + octaveOffset + offset);
    }
  });

  document.addEventListener('keyup', e => {
    const offset = KEY_TO_SEMITONE[e.key.toLowerCase()];
    if (offset !== undefined) {
      held.delete(e.key);
      releaseNote(60 + octaveOffset + offset);
    }
  });

  // Panic on Escape
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape') { engine.allNotesOff(); activeNotes.clear(); held.clear(); updateKeyVisuals(); }
  });

  document.getElementById('oct-down').addEventListener('click', () => {
    octaveOffset = Math.max(-48, octaveOffset - 12);
    updateOctaveLabel();
    buildKeyboard();
  });
  document.getElementById('oct-up').addEventListener('click', () => {
    octaveOffset = Math.min(48, octaveOffset + 12);
    updateOctaveLabel();
    buildKeyboard();
  });
}

function updateOctaveLabel() {
  const oct = 4 + Math.round(octaveOffset / 12);
  const el = document.getElementById('octave-label');
  if (el) el.textContent = `OCT ${oct}`;
}

function updateKeyVisuals() {
  document.querySelectorAll('.piano-key.pressed').forEach(k => k.classList.remove('pressed'));
}

// ═══════════════════════════════════════════════════════════
// WEB MIDI
// ═══════════════════════════════════════════════════════════
// Full CC→param-ID map, mirrors engine.rs CC routing exactly.
// Used to keep knob UI in sync when a CC arrives.
const MIDI_CC_MAP = {
  // Mixer / Output
   7: P.MASTER_VOL,  10: P.OSC_MIX,     15: P.NOISE,
  // Filter
  74: P.FILT_CUT,    71: P.FILT_RES,    16: P.FILT_ENV,   14: P.FILT_KEY,
  // Amp envelope
  73: P.AMP_ATK,     75: P.AMP_DEC,     76: P.AMP_SUS,    72: P.AMP_REL,
  // Filter envelope
  77: P.FLT_ATK,     78: P.FLT_DEC,     79: P.FLT_SUS,    80: P.FLT_REL,
  // LFO
  17: P.LFO_RATE,    18: P.LFO_DEPTH,   24: P.LFO_OSC,    25: P.LFO_FILT,  26: P.LFO_PW,
  // Oscillators
  20: P.OSC1_FINE,   21: P.OSC1_PW,     22: P.OSC2_COARSE, 23: P.OSC2_FINE, 27: P.OSC2_PW,
  // Poly Mod
  28: P.PM_FREQ,     29: P.PM_PW,       30: P.PM_FILT,
  // Character
  19: P.SLOP,        31: P.UNISON_DET,
};

function handleMidiMessage(e) {
  const [status, d1, d2] = e.data;

  // Forward the raw bytes to the Rust engine — it handles all routing internally.
  if (engine.wasm) engine.wasm.midi_message(status, d1, d2);

  const type = status & 0xF0;

  // Keep the JS engine state + knob UI in sync for note and CC events.
  switch (type) {
    case 0x90: d2 > 0 ? triggerNote(d1) : releaseNote(d1); break;
    case 0x80: releaseNote(d1);                              break;
    case 0xB0: {
      // Sustain (CC 64) and mod wheel (CC 1) are handled by the Rust engine
      // via midi_message(); just sync knob UI for mapped CCs.
      const id = MIDI_CC_MAP[d1];
      if (id !== undefined) {
        const n = d2 / 127;
        if (allKnobs.has(id)) allKnobs.get(id).setValue(n);
      }
      break;
    }
    // Pitch bend is fully handled by the Rust engine via midi_message().
    // No JS-side state update needed.
  }
}

async function initWebMidi() {
  if (!navigator.requestMIDIAccess) return;
  try {
    const midi = await navigator.requestMIDIAccess();
    const attachAll = () => {
      midi.inputs.forEach(inp => inp.onmidimessage = handleMidiMessage);
      document.getElementById('midi-indicator').classList.toggle('on', midi.inputs.size > 0);
    };
    attachAll();
    midi.onstatechange = attachAll;
  } catch (err) {
    console.info('[Premonition] Web MIDI unavailable:', err.message);
  }
}

// ═══════════════════════════════════════════════════════════
// OSCILLOSCOPE
// ═══════════════════════════════════════════════════════════
function startOscilloscope() {
  const canvas = document.getElementById('oscilloscope');
  const ctx    = canvas.getContext('2d');
  const W = canvas.width;
  const H = canvas.height;
  const midY = H / 2;

  // Fake animated waveform for when WASM isn't loaded
  let fakePhase = 0;

  function draw() {
    requestAnimationFrame(draw);

    // Background
    ctx.fillStyle = 'rgba(8, 8, 11, 0.85)';
    ctx.fillRect(0, 0, W, H);

    // Centre line
    ctx.strokeStyle = 'rgba(70, 70, 100, 0.4)';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(0, midY); ctx.lineTo(W, midY);
    ctx.stroke();

    const hasWasm = !!engine.wasm;
    const running  = engine.running;

    ctx.lineWidth   = 1;
    ctx.strokeStyle = hasWasm ? '#4ade80' : 'rgba(74, 222, 128, 0.22)';
    ctx.beginPath();

    if (hasWasm && running) {
      // Real waveform from audio buffer
      const buf = engine.oscBuf;
      const len = buf.length;
      let wp = engine.oscWrPos;
      
      // Compute range based on currently active notes (if any)
      let highestNote = -1;
      for (let n of activeNotes) highestNote = Math.max(highestNote, n);
      let freq = 110.0; // Default rendering freq
      if (highestNote > 0) {
        freq = 440.0 * Math.pow(2, (highestNote - 69) / 12);
      }
      
      const sr = engine.audioCtx.sampleRate;
      // Draw 3 cycles of the fundamental (capped safely)
      const samplesToDraw = Math.min(len, Math.max(20, Math.floor(3 * sr / freq)));
      
      for (let x = 0; x < W; x++) {
        // Find position tracing backward from write pos so we see latest stable waveform
        const offset = Math.floor((1 - x / W) * samplesToDraw);
        const idx = (wp - offset + len) % len;
        const y   = midY - buf[idx] * (midY - 3);
        x === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
    } else {
      // Animated demo sine when idle
      fakePhase += 0.04;
      for (let x = 0; x < W; x++) {
        const t = x / W;
        const y = midY - Math.sin(t * Math.PI * 6 + fakePhase) * (midY * 0.4)
                        * (0.5 + 0.5 * Math.sin(t * Math.PI * 2 + fakePhase * 0.3));
        x === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.strokeStyle = 'rgba(74, 222, 128, 0.18)';
    }
    ctx.stroke();

    // Scanline CRT effect
    ctx.fillStyle = 'rgba(0, 0, 0, 0.06)';
    for (let y = 0; y < H; y += 3) ctx.fillRect(0, y, W, 1);
  }
  draw();
}

// ═══════════════════════════════════════════════════════════
// ACTIVATE BUTTON
// ═══════════════════════════════════════════════════════════
function initActivateButton() {
  const btn = document.getElementById('start-audio');
  btn.addEventListener('click', async () => {
    if (engine.running) return;
    btn.textContent = '⌛ STARTING…';
    btn.disabled = true;
    try {
      await engine.start();
      await initWebMidi();
      btn.textContent = '● RUNNING';
      btn.classList.add('running');
    } catch (err) {
      btn.textContent = '✕ ERROR';
      btn.disabled = false;
      console.error('[Premonition] Audio start failed:', err);
    }
  });
}

// ═══════════════════════════════════════════════════════════
// UTILITY
// ═══════════════════════════════════════════════════════════
const clamp01 = v => Math.max(0, Math.min(1, v));

// ═══════════════════════════════════════════════════════════
// INIT
// ═══════════════════════════════════════════════════════════
document.addEventListener('DOMContentLoaded', () => {
  initControls();
  buildKeyboard();
  initKeyboardInput();
  initPresetBar();
  initActivateButton();
  startOscilloscope();

  // Load init preset to set all knobs to their correct positions
  loadPreset('init');

  console.info('[Premonition] UI ready. Click ACTIVATE to start audio.');
});
