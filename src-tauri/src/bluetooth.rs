use btleplug::{
    api::Manager as _,
    platform::{Adapter, Manager},
    Error,
};

use self::discovery::Discovery;

pub mod discovery;

pub(crate) async fn adapter() -> Result<Adapter, Error> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    adapters
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotSupported("No Bluetooth adapters found".to_string()))
}

#[derive(Clone)]
pub(crate) struct Bluetooth {
    adapter: Adapter,
    pub discovery: Discovery,
}

impl Bluetooth {
    pub async fn new() -> Self {
        let adapter = adapter().await.expect("Bluetooth not found");
        let discovery = Discovery::start(adapter.clone());

        Self { adapter, discovery }
    }
}
