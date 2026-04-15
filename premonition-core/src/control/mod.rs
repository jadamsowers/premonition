//! Control and parameter modules.

mod midi;
mod params;
mod smoothing;

pub use midi::MidiMessage;
pub use params::{ParameterId, Parameters};
pub use smoothing::ParameterSmoother;
