# Prophet UI / UX Design Specification

1. Core Design Philosophy
“One knob per function”
No menus, no hidden parameters
Everything visible at once
Layout mirrors signal flow (left → right)
Tone:
Professional, instrument-like (not “software-y”)
Slightly vintage, but not decorative or cluttered
Designed for tactile intuition
2. Layout Structure
Horizontal Panel Layout
Divide UI into vertical sections:

[ Osc A ] [ Osc B ] [ Mixer ] [ Filter ] [ Env ] [ Env ] [ LFO / Mod ]
Each section is a clearly separated “module”
Equal visual weight, aligned in a grid
Labels at top of each section
Visual Grouping
Use boxed sections or subtle panel divisions
Each module has:
Title
Knobs grouped logically
Minimal spacing between related controls
3. Controls
Knobs (Primary Control)
Circular rotary knobs
Large, readable, evenly spaced
Indicator line or notch (high contrast)
Behavior:
Linear rotation (approx. 270° sweep)
Smooth, slightly damped response
Optional fine-adjust (shift/alt modifier)
Visual style:
Matte finish
Subtle shadow or bevel (hardware feel)
No glossy or skeuomorphic excess
Switches / Toggles
Used for:
Waveform selection
On/off states
Style:
Simple rocker or vertical toggle
Clearly binary (no ambiguity)
Sliders (Optional)
Rare in original; prefer knobs
If used, keep minimal and consistent
4. Color & Material
Background
Dark panel (charcoal / black / deep brown)
Slight texture (brushed metal or matte paint)
Accent Colors
Use sparingly and consistently:
White: labels and primary markings
Orange / red: highlights (filter, key controls)
Blue or green: secondary modulation (optional)
Text
All caps or small caps
Clean sans-serif or slightly industrial font
High contrast, no decorative fonts
5. Labeling
Every control labeled directly (no tooltips required)
Labels are:
Short (e.g., “CUTOFF”, “RESONANCE”)
Positioned above or below knob
Section headers slightly larger
6. Signal Flow Visualization
Implicit, not explicit
Achieved through layout order:
Oscillators → Mixer → Filter → Amp
Optional:
Thin lines or spacing cues between sections
7. Interaction Feel
Responsiveness
Immediate parameter response (no lag)
Smooth interpolation (no stepping)
Feedback
Knob position = state (no extra UI needed)
Optional numeric readout on hover/drag
8. Analog Character (Important)
Subtle imperfections:
Slight knob position jitter (very minimal, optional)
Tiny inconsistencies in spacing/alignment (simulated)
Non-perfect gradients or textures
Avoid:
Perfect flat UI (too digital)
Overly glossy or 3D-heavy skeuomorphism
9. Scaling for Software
Maintain Proportions
Keep original panel aspect ratio
Scale uniformly
Responsive Rules
Knobs scale proportionally
Spacing remains consistent
Avoid collapsing into menus
10. Optional Modern Additions (Tasteful)
If extending beyond original:
Patch selector (top bar)
Subtle preset browser
Hidden advanced panel (not on main surface)
MIDI learn overlay
Keep these:
Secondary to main panel
Visually separated from core controls