use btleplug::api::Characteristic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacteristicData {
    pub uuid: String,
}

impl From<&Characteristic> for CharacteristicData {
    fn from(characteristic: &Characteristic) -> Self {
        Self {
            uuid: characteristic.uuid.to_string(),
        }
    }
}
