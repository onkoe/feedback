pub trait Checksum {
    /// Calculates the checksum of this `Message`.
    fn checksum(&self) -> u8;

    /// Check if the checksum is correct.
    fn is_checksum_correct(&self) -> bool;
}
