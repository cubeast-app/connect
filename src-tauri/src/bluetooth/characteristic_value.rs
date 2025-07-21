#[derive(Debug)]
pub struct CharacteristicValue {
    /// Represents the time in milliseconds since the Unix epoch.
    pub timestamp: u64,
    pub value: Vec<u8>,
}
