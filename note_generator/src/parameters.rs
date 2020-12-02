use std::sync::Mutex;
use util::parameter_value_conversion::{bool_to_f32, byte_to_f32, f32_to_bool, f32_to_byte};
use util::HostCallbackLock;
use vst::plugin::{HostCallback, PluginParameters};
use vst::util::ParameterTransfer;
use util::constants::{NOTE_NAMES, C0};

pub struct NoteGeneratorPluginParameters {
    pub host: Mutex<HostCallbackLock>,
    pub transfer: ParameterTransfer,
}

#[repr(i32)]
pub enum Parameter {
    Channel = 0,
    Pitch,
    Velocity,
    NoteOffVelocity,
    Pressure,
    Trigger,
    TriggeredPitch,
    TriggeredChannel
}


impl From<i32> for Parameter {
    fn from(i: i32) -> Self {
        match i {
            0 => Parameter::Channel,
            1 => Parameter::Pitch,
            2 => Parameter::Velocity,
            3 => Parameter::NoteOffVelocity,
            4 => Parameter::Pressure,
            5 => Parameter::Trigger,
            6 => Parameter::TriggeredPitch,
            7 => Parameter::TriggeredChannel,
            _ => panic!(format!("No such Parameter {}", i))
        }
    }
}


impl NoteGeneratorPluginParameters {
    #[inline]
    fn set_byte_parameter(&self, index: Parameter, value: u8) {
        self.transfer
            .set_parameter(index as usize, byte_to_f32(value))
    }

    #[inline]
    pub fn get_byte_parameter(&self, index: Parameter) -> u8 {
        f32_to_byte(self.transfer.get_parameter(index as usize))
    }

    #[inline]
    pub fn get_bool_parameter(&self, index: Parameter) -> bool {
        f32_to_bool(self.transfer.get_parameter(index as usize))
    }

    #[inline]
    fn set_bool_parameter(&self, index: Parameter, value: bool) {
        self.transfer
            .set_parameter(index as usize, bool_to_f32(value))
    }

    #[inline]
    fn get_displayable_channel(&self) -> u8 {
        // NOT the stored value, but the one used to show on the UI
        self.get_byte_parameter(Parameter::Channel) / 8 + 1
    }

    fn get_pitch_label(&self) -> String {
        format!(
            "{}{}",
            NOTE_NAMES[self.get_byte_parameter(Parameter::Pitch) as usize % 12],
            ((self.get_byte_parameter(Parameter::Pitch) as i8) - C0) / 12
        )
    }

    #[inline]
    fn get_velocity(&self) -> u8 {
        self.get_byte_parameter(Parameter::Velocity)
    }

    #[inline]
    fn get_note_off_velocity(&self) -> u8 {
        self.get_byte_parameter(Parameter::NoteOffVelocity)
    }

    #[inline]
    fn get_pressure(&self) -> u8 {
        self.get_byte_parameter(Parameter::Pressure)
    }

    #[inline]
    fn get_trigger(&self) -> bool {
        self.get_bool_parameter(Parameter::Trigger)
    }

    pub fn copy_parameter(&self, from_index: Parameter, to_index: Parameter) {
        self.set_parameter(to_index as i32, self.get_parameter(from_index as i32));
    }

    pub fn new(host: HostCallback) -> Self {
        NoteGeneratorPluginParameters {
            host: Mutex::new(HostCallbackLock { host }),
            ..Default::default()
        }
    }
}

impl PluginParameters for NoteGeneratorPluginParameters {
    fn get_parameter_text(&self, index: i32) -> String {
        match Parameter::from(index as i32) {
            Parameter::Channel => format!("{}", self.get_displayable_channel()),
            Parameter::Pitch => self.get_pitch_label(),
            Parameter::Velocity => format!("{}", self.get_velocity()),
            Parameter::NoteOffVelocity => format!("{}", self.get_note_off_velocity()),
            Parameter::Pressure => format!("{}", self.get_pressure()),
            Parameter::Trigger => format!("{}", self.get_trigger()),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match Parameter::from(index as i32) {
            Parameter::Channel => "Channel",
            Parameter::Pitch => "Pitch",
            Parameter::Velocity => "Velocity",
            Parameter::NoteOffVelocity => "Note off velocity",
            Parameter::Pressure => "Pressure",
            Parameter::Trigger => "Trigger generated note",
            _ => "",
        }
        .to_string()
    }

    fn get_parameter(&self, index: i32) -> f32 {
        self.transfer.get_parameter(index as usize)
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match Parameter::from(index as i32) {
            Parameter::Trigger => {
                // boolean case: in order to ignore intermediary changes,
                // don't just pass the unchanged f32
                let new_value = f32_to_bool(value);
                let old_value = self.get_bool_parameter(Parameter::Trigger);

                if new_value != old_value {
                    self.set_bool_parameter(Parameter::Trigger, new_value)
                }
            }
            _ => {
                // reduce to a byte and compare, so modulators don't generate tons of
                // irrelevant changes
                let new_value = f32_to_byte(value);
                let old_value = self.get_byte_parameter(Parameter::from(index));
                if new_value != old_value {
                    self.set_byte_parameter(Parameter::from(index), new_value)
                }
            }
        }
    }

    fn string_to_parameter(&self, index: i32, text: String) -> bool {
        // actually never called in bitwig
        match Parameter::from(index as i32) {
            Parameter::Channel => match text.parse::<u8>() {
                Ok(n) => {
                    if n > 0 && n <= 16 {
                        self.set_byte_parameter(Parameter::from(index), n);
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            },
            Parameter::Velocity | Parameter::NoteOffVelocity | Parameter::Pressure => match text.parse::<u8>() {
                Ok(n) => {
                    if n < 128 {
                        self.set_byte_parameter(Parameter::Velocity, n);
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            },
            Parameter::Pitch => match NOTE_NAMES.iter().position(|&s| text.starts_with(s)) {
                None => false,
                Some(position) => {
                    match text[NOTE_NAMES[position].len()..text.len()].parse::<i8>() {
                        Ok(octave) => {
                            if octave >= -2 && octave <= 8 {
                                let pitch = octave as i16 * 12 + C0 as i16 + position as i16;
                                if pitch < 128 {
                                    self.set_byte_parameter(Parameter::Pitch, pitch as u8);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        Err(_) => false,
                    }
                }
            },
            Parameter::Trigger => match text.to_ascii_lowercase().as_ref() {
                "0" | "off" | "" => {
                    self.set_bool_parameter(Parameter::Trigger, false);
                    true
                }
                "1" | "on" => {
                    self.set_bool_parameter(Parameter::Trigger, true);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn get_preset_data(&self) -> Vec<u8> {
        (0..8).map(|i| self.get_byte_parameter(Parameter::from(i))).collect()
    }

    fn get_bank_data(&self) -> Vec<u8> {
        (0..8).map(|i: i32| self.get_byte_parameter(Parameter::from(i))).collect()
    }

    fn load_preset_data(&self, data: &[u8]) {
        for (i, item) in data.iter().enumerate() {
            self.set_byte_parameter(Parameter::from(i as i32), *item);
        }
    }

    fn load_bank_data(&self, data: &[u8]) {
        for (i, item) in data.iter().enumerate() {
            self.set_byte_parameter(Parameter::from(i as i32), *item);
        }
    }
}

impl Default for NoteGeneratorPluginParameters {
    fn default() -> Self {
        let parameters = NoteGeneratorPluginParameters {
            host: Default::default(),
            transfer: ParameterTransfer::new(8),
        };
        parameters.set_byte_parameter(Parameter::Pitch, C0 as u8);
        parameters.set_byte_parameter(Parameter::Velocity, 64);
        parameters
    }
}