use btleplug::api::Manager as _;
use btleplug::platform::{Adapter, Manager};
use btleplug::Error;

pub async fn bluetooth_adapter() -> Result<Adapter, Error> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    adapters
        .into_iter()
        .nth(0)
        .ok_or_else(|| Error::NotSupported("No Bluetooth adapters found".to_string()))
}
