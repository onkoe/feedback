//! # Parse
//!
//! A module that parses a given slice into a valid message.

use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{error::ParsingError, Arm, Led, Science, Wheels};

/// Any kind of message that should be sent to/from the rover.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Wheels(Wheels),
    Led(Led),
    Arm(Arm),
    Science(Science),
}

/// A PyO3-friendly version of the `Message` enum.
#[doc(hidden)]
#[pyclass]
#[derive(Debug, Clone, Copy)]
pub enum PyMessage {
    Wheels { wheels: Wheels },
    Led { led: Led },
    Arm { arm: Arm },
    Science { science: Science },
}

impl PyMessage {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

/// this is some nonsense... but it's required nonsense.
/// see [pyo3 issue #3748](https://github.com/PyO3/pyo3/issues/3748) for info
impl From<PyMessage> for Message {
    fn from(val: PyMessage) -> Self {
        match val {
            PyMessage::Wheels { wheels } => Message::Wheels(wheels),
            PyMessage::Led { led } => Message::Led(led),
            PyMessage::Arm { arm } => Message::Arm(arm),
            PyMessage::Science { science } => Message::Science(science),
        }
    }
}

/// again. nonsense
impl From<Message> for PyMessage {
    fn from(val: Message) -> Self {
        match val {
            Message::Wheels(wheels) => PyMessage::Wheels { wheels },
            Message::Led(led) => PyMessage::Led { led },
            Message::Arm(arm) => PyMessage::Arm { arm },
            Message::Science(science) => PyMessage::Science { science },
        }
    }
}

/// Parse an input slice into a valid message.
/// ```
/// # use feedback::parse::parse;
/// #
/// assert!(parse(&[0x09]).is_err());
/// ```
pub fn parse(input: &[u8]) -> Result<Message, ParsingError> {
    let input_len = input.len() as u32;

    // check if we have a subsystem byte
    if input_len < 1 {
        return Err(ParsingError::ZeroLengthSlice);
    }

    // we do! let's match on it
    let subsystem = input[0];

    match subsystem {
        Wheels::SUBSYSTEM_BYTE => {
            // you have to specify the part byte
            if input_len < 2 {
                return Err(ParsingError::NoEboxPart);
            }

            // it exists! now we can parse it
            // parse the second byte, input[1], to tell if it's wheel or leds
            let part = input[1];
            match part {
                // wheel part
                Wheels::PART_BYTE => {
                    check_length(input_len, subsystem, part, 9)?;

                    Ok(Message::Wheels(Wheels::new(
                        input[2], input[3], input[4], input[5], input[6], input[7], input[8],
                    )))
                }

                // leds part
                Led::PART_BYTE => {
                    check_length(input_len, subsystem, part, 5)?;

                    Ok(Message::Led(Led {
                        red: input[2],
                        green: input[3],
                        blue: input[4],
                    }))
                }

                malformed_part => {
                    // invalid input
                    Err(ParsingError::InvalidSubsystem(malformed_part))
                }
            }
        }

        Arm::SUBSYSTEM_BYTE => {
            check_length(input_len, subsystem, 0x00, 8)?;

            let arm = Arm {
                bicep: input[1],
                forearm: input[2],
                base: input[3],
                wrist_pitch: input[4],
                wrist_roll: input[5],
                claw: input[6],
                checksum: input[7],
            };

            Ok(Message::Arm(arm))
        }

        Science::SUBSYSTEM_BYTE => {
            // the given slice was malformed. 😖
            check_length(input_len, subsystem, 0x0, 7)?;

            let sci = Science {
                big_actuator: input[1],
                drill: input[2],
                small_actuator: input[3],
                test_tubes: input[4],
                camera_servo: input[5],
                checksum: input[6],
            };

            Ok(Message::Science(sci))
        }

        // otherwise, we got invalid input
        malformed_subsys => Err(ParsingError::InvalidSubsystem(malformed_subsys)),
    }
}

/// Parse an input slice into a valid message.
#[pyfunction(name = "parse")]
pub fn pyparse(input: &[u8]) -> PyResult<PyMessage> {
    parse(input)
        .map_err(|e| PyValueError::new_err(e.to_string()))
        .map(|t| t.into())
}

/// Checks if the given input length is equal to the expected length. If so, returns `Ok(())`.
/// Otherwise, returns a `ParsingError::LengthInconsistency` error.
///
/// This avoids some kinda annoying boilerplate in the `parse` function.
/// (please stabilize #74935 🥹)
const fn check_length(
    input_len: u32,
    subsystem: u8,
    part: u8,
    expected: u32,
) -> Result<(), ParsingError> {
    if input_len != expected {
        Err(ParsingError::LengthInconsistency {
            subsystem,
            part,
            length: input_len,
            expected_length: expected,
        })
    } else {
        Ok(())
    }
}
