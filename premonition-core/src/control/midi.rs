//! MIDI message types.

#[derive(Clone, Copy, Debug)]
pub enum MidiMessage {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    PitchBend {
        channel: u8,
        value: f32,
    },
    ModWheel {
        channel: u8,
        value: f32,
    },
    SustainOn {
        channel: u8,
    },
    SustainOff {
        channel: u8,
    },
    Aftertouch {
        channel: u8,
        note: u8,
        value: f32,
    },
    AllNotesOff {
        channel: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
}

impl MidiMessage {
    pub fn from_bytes(status: u8, data1: u8, data2: u8) -> Option<Self> {
        let channel = status & 0x0F;
        let message_type = status & 0xF0;

        match message_type {
            0x80 => Some(MidiMessage::NoteOff {
                channel,
                note: data1,
            }),
            0x90 => {
                if data2 > 0 {
                    Some(MidiMessage::NoteOn {
                        channel,
                        note: data1,
                        velocity: data2,
                    })
                } else {
                    Some(MidiMessage::NoteOff {
                        channel,
                        note: data1,
                    })
                }
            }
            0xA0 => Some(MidiMessage::Aftertouch {
                channel,
                note: data1,
                value: data2 as f32 / 127.0,
            }),
            0xB0 => Some(Self::parse_control_change(channel, data1, data2)),
            0xE0 => {
                let bend_value = ((data2 as u16) << 7) | (data1 as u16);
                let normalized = (bend_value as f32 - 8192.0) / 8192.0;
                Some(MidiMessage::PitchBend {
                    channel,
                    value: normalized,
                })
            }
            _ => None,
        }
    }

    fn parse_control_change(channel: u8, controller: u8, value: u8) -> MidiMessage {
        match controller {
            1 => MidiMessage::ModWheel {
                channel,
                value: value as f32 / 127.0,
            },
            64 => {
                if value >= 64 {
                    MidiMessage::SustainOn { channel }
                } else {
                    MidiMessage::SustainOff { channel }
                }
            }
            _ => MidiMessage::ControlChange {
                channel,
                controller,
                value,
            },
        }
    }
}
