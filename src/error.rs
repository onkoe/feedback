use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, PartialOrd)]
pub enum ParsingError {
    #[error("You must supply a slice with a length greater than zero.")]
    ZeroLengthSlice,
    #[error("First byte doesn't code for a valid subsystem. Given: `{0:x}`.")]
    InvalidSubsystem(u8),
    #[error("The ebox subsystem must be given a second byte, input[1], to specify which part to control. None was given.")]
    NoEboxPart,
    #[error(
        "
        The given slice wasn't the correct length for subsystem \
        `{subsystem:x}` on part {part:x}. Expected a slice of length 9, but got a \
        slice of length `{length}`.
    "
    )]
    LengthInconsistency {
        subsystem: u8,
        part: u8,
        length: u32,
    },
    #[error("The given slice was malformed.")]
    MalformedMessage,
}
