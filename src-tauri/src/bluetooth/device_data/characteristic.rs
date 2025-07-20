use btleplug::api::{CharPropFlags, Characteristic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacteristicData {
    pub uuid: String,
    pub read: bool,
    pub write: bool,
    pub notify: bool,
}

impl From<&Characteristic> for CharacteristicData {
    fn from(characteristic: &Characteristic) -> Self {
        Self {
            uuid: characteristic.uuid.to_string(),
            read: characteristic.properties & CharPropFlags::READ != CharPropFlags::empty(),
            write: characteristic.properties & CharPropFlags::WRITE != CharPropFlags::empty(),
            notify: characteristic.properties & CharPropFlags::NOTIFY != CharPropFlags::empty(),
        }
    }
}
