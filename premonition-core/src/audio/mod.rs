//! Audio processing modules.

mod engine;
mod envelope;
mod filter;
mod imperfection;
mod lfo;
mod mixer;
mod noise;
mod oscillator;
mod output;
mod poly_mod;
mod vca;
mod voice;
mod voice_manager;

pub use crate::effects::EffectsChain;
pub use engine::Engine;
pub use envelope::Envelope;
pub use filter::Filter;
pub use imperfection::Imperfection;
pub use lfo::Lfo;
pub use mixer::Mixer;
pub use noise::Noise;
pub use oscillator::{Oscillator, Waveform};
pub use output::OutputStage;
pub use poly_mod::PolyMod;
pub use vca::Vca;
pub use voice::Voice;
pub use voice_manager::VoiceManager;
