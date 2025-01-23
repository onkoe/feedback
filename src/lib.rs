//! # Feedback
//!
//! A library crate that encodes/decodes messages to or from the Rover.
//!
//! ## Protocol
//!
//! The protocol is as follows:
//!
//! ### Wheels
//!
//! [u8; 9]: [0x01 (wheels subsystem), 0x01 (wheels part), ]
//!
//! subsystem byte, part byte (optional); etc.

use pyo3::prelude::*;

pub mod checksum;
pub mod error;
pub mod parse;
pub mod send;

pub mod prelude {
    pub use super::send::RoverController;
}

/// For the Rover, the Wheels struct represents the current state of each of the six wheels.
/// Each `wheelx` value is a u8, with the neutral position being 126.
#[pyclass]
#[derive(Debug, Clone, Copy)]
pub struct Wheels {
    pub wheel0: u8,
    pub wheel1: u8,
    pub wheel2: u8,
    pub wheel3: u8,
    pub wheel4: u8,
    pub wheel5: u8,
    /// The sum of the current wheel speeds.
    pub checksum: u8,
}

#[pymethods]
impl Wheels {
    pub const SUBSYSTEM_BYTE: u8 = 0x01;
    pub const PART_BYTE: u8 = 0x01;

    /// The motor value at which a motor isn't moving.
    pub const NEURTAL_SPEED: u8 = 126;

    /// Creates a new `Wheels` object from the given values. Unchecked.
    #[new]
    pub const fn new(
        wheel0: u8,
        wheel1: u8,
        wheel2: u8,
        wheel3: u8,
        wheel4: u8,
        wheel5: u8,
        checksum: u8,
    ) -> Self {
        Self {
            wheel0,
            wheel1,
            wheel2,
            wheel3,
            wheel4,
            wheel5,
            checksum,
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

/// The flashing LED on the top of the Rover
#[pyclass]
#[derive(Debug, Clone, Copy)]
pub struct Led {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[pymethods]
impl Led {
    pub const SUBSYSTEM_BYTE: u8 = 0x01;
    pub const PART_BYTE: u8 = 0x02;

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

/// The little robotic arm on the sticking out of the Rover
/// old capstooOOOone
#[pyclass]
#[derive(Debug, Clone, Copy)]
pub struct Arm {
    pub bicep: u8,
    pub forearm: u8,
    pub base: u8,
    pub wrist_pitch: u8,
    pub wrist_roll: u8,
    pub claw: u8,
    pub checksum: u8,
}

#[pymethods]
impl Arm {
    pub const SUBSYSTEM_BYTE: u8 = 0x02;

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

/// The science package on the Rover, including the utilities needed to perform
/// field experiments.
#[pyclass]
#[derive(Debug, Clone, Copy)]
pub struct Science {
    big_actuator: u8,
    drill: u8,
    small_actuator: u8,
    test_tubes: u8,
    camera_servo: u8,
    checksum: u8,
}

#[pymethods]
impl Science {
    pub const SUBSYSTEM_BYTE: u8 = 0x03;

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pymodule]
fn feedback(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Wheels>()?;
    m.add_class::<Led>()?;
    m.add_class::<Arm>()?;
    m.add_class::<Science>()?;
    m.add_class::<parse::PyMessage>()?;
    m.add_function(wrap_pyfunction!(parse::pyparse, m)?)?;
    Ok(())
}
