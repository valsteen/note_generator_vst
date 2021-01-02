use log::info;

use util::raw_message::RawMessage;
use util::messages::{NoteOn, Timbre, Pressure, PitchBend, AfterTouch};


pub struct ExpressiveNote {
    pub channel: u8,
    pub pitch: u8,
    pub velocity: u8,
    pub pressure: u8,
    pub timbre: u8,
    pub pitchbend: i32,
}


impl ExpressiveNote {
    #[cfg(not(use_channel_pressure))]
    #[inline]
    fn get_pressure_note(&self) -> RawMessage {
        AfterTouch {
            channel: self.channel,
            pitch: self.pitch,
            value: self.pressure,
        }.into()
    }

    #[cfg(use_channel_pressure)]
    #[inline]
    fn get_pressure_note(&self) -> RawMessage {
        Pressure {
            channel: self.channel,
            value: self.pressure,
        }.into()
    }

    pub fn into_rawmessages(self) -> Vec<RawMessage> {
        vec![
            NoteOn {
                channel: self.channel,
                pitch: self.pitch,
                velocity: self.velocity, // todo mixing between pattern and note
            }.into(),
            self.get_pressure_note(),
            Timbre {
                channel: self.channel,
                value: self.timbre,
            }.into(),
            PitchBend {
                channel: self.channel,
                millisemitones: self.pitchbend,
            }.into(),
        ]
    }
}