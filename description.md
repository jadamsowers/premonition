Prophet‑Style Synth Engine (Realistic Analog‑Inspired Specification)
1. Overall Architecture
Polyphonic analog‑modeled subtractive synthesizer.
Voices: Fixed polyphony (5 or 8), not dynamically scalable.
Each voice contains slightly mismatched components to emulate calibration tolerances.
Signal Flow (per voice):  
Osc1 + Osc2 + Noise → Mixer (mild nonlinearities) → 24 dB LPF (nonlinear) → VCA (OTA‑style) → Output
Control signals (envs, LFO, Poly‑Mod) are not perfectly stable and include small timing jitter.
2. Oscillator Section (Per Voice)
Voltage‑controlled oscillator models with drift, temperature‑like instability, and imperfect tuning.
VCO1
Waveforms: Saw, Pulse (PW not perfectly symmetrical).
Controls:
Frequency: Slight quantization/rounding to emulate analog scaling.
Fine tune: Limited range (±7 semitones typical).
Pulse width: Nonlinear response near extremes.
Modulation:
LFO → pitch (with slight delay/lag).
Envelope → pitch (limited depth).
Hard sync to VCO2 (reset not sample‑accurate; produces analog‑style tearing).
VCO2
Waveforms: Saw, Pulse, optional Triangle (triangle is not perfectly pure).
Controls:
Coarse tune: Semitone steps with slight detune error.
Fine tune: Imperfect scaling.
PW: Slightly different response curve than VCO1.
Modulation:
LFO → pitch
Audio‑rate modulation of Osc1 or filter (bandwidth‑limited, not true FM).
Sync behavior varies slightly per voice.
3. Noise Generator
White noise with slight pink tilt and level variance between voices.
Noise amplitude is not perfectly stable.
4. Mixer Section
Three inputs: Osc1, Osc2, Noise.
Each channel has gain staging that saturates slightly when levels exceed ~80%.
Summing is not perfectly linear; introduces mild harmonic distortion and intermodulation.
5. Filter Section
Curtis‑style or SSM‑style 4‑pole low‑pass filter model.
Characteristics:
Cutoff frequency varies per voice (component tolerance).
Resonance peak is nonlinear and shifts slightly with input level.
Self‑oscillation amplitude is not perfectly stable.
Keyboard tracking is approximate, not mathematically exact.
Filter envelope amount interacts with input level (envelope depth feels different at high resonance).
6. Amplifier (VCA)
OTA‑style VCA with level‑dependent distortion.
Envelope‑driven, but envelope‑to‑VCA response is not perfectly linear.
Optional velocity sensitivity (not present on early Prophets).
7. Envelopes (Per Voice)
Two ADSR envelopes with analog‑style timing.
Characteristics:
Attack/decay/release curves are exponential but vary slightly per voice.
Minimum attack is not instantaneous (~2–4 ms).
Envelope retriggering is not sample‑accurate; slight timing jitter.
Sustain level may differ slightly between voices.
8. LFO
Single global LFO (per‑voice LFOs are not historically accurate).
Waveforms: Triangle, Saw, Square, Random (S&H).
Rate: Slight internal drift; not perfectly stable.
Modulation depth is not linear across the full range.
Destinations: Osc pitch, PW, filter cutoff.
9. Poly‑Mod Section
Sources:
Osc2 (audio‑rate, bandwidth‑limited)
Filter envelope (with slight scaling error)
Destinations:
Osc1 frequency
Osc1 pulse width
Filter cutoff
Characteristics:
Modulation depth is not symmetrical (positive vs negative).
Audio‑rate modulation produces analog‑style sidebands, not clean FM.
10. Voice Allocation
Round‑robin with imperfect voice reset behavior.
Voice stealing is not perfectly smooth; envelopes may click if stolen mid‑cycle.
Unison mode stacks voices with natural detune from oscillator drift.
11. Analog Imperfections (Critical for Authenticity)
Per‑voice randomization:
Oscillator drift (slow, chaotic, not purely random).
Filter cutoff variance (fixed per voice).
Envelope timing offsets.
Mixer gain differences.
Slight DC offsets.
User‑controlled “Slop” parameter exaggerates these behaviors.
12. Output Stage
Summed voices pass through a soft‑clipping stage.
Output is slightly noisy and not perfectly symmetrical.
Optional stereo spread (not original but common in modern emulations).
13. Optional Modern Enhancements
Velocity → filter/amp
Aftertouch → pitch/filter
Effects (chorus, delay, reverb)
MPE support
All enhancements should be bypassable to preserve authentic Prophet behavior.
Key Prophet Characteristics to Preserve
Oscillators that drift, beat, and misbehave slightly.
Filter that saturates, shifts, and interacts with input level.
Snappy but imperfect envelopes.
Poly‑Mod that feels raw and unstable.
Voice‑to‑voice variation that makes chords alive.