use btleplug::api::Service;
use serde::{Deserialize, Serialize};

use super::characteristic::CharacteristicData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceData {
    pub uuid: String,
    pub characteristics: Vec<CharacteristicData>,
}

impl From<&Service> for ServiceData {
    fn from(service: &Service) -> Self {
        let characteristics = service.characteristics.iter().map(|c| c.into()).collect();

        Self {
            uuid: service.uuid.to_string(),
            characteristics,
        }
    }
}
