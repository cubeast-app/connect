use std::pin::Pin;

use btleplug::{
    api::{Central as _, CentralEvent, Peripheral as _},
    platform::Adapter,
    Error,
};
use futures_util::{
    stream::{iter, select},
    Stream, StreamExt as _,
};
use tracing::{debug, warn};

use super::discovered_device::DiscoveredDevice;

pub type DiscoveryStream = Pin<Box<dyn Stream<Item = Vec<DiscoveredDevice>> + Send>>;

pub(super) async fn discovery_stream(
    adapter: Adapter,
    initial: Vec<DiscoveredDevice>,
) -> Result<DiscoveryStream, Error> {
    let events = adapter.events().await?;

    let events = events.filter_map(move |event| {
        let adapter = adapter.clone();

        async move {
            match event.clone() {
                CentralEvent::DeviceDiscovered(_)
                | CentralEvent::DeviceUpdated(_)
                | CentralEvent::ManufacturerDataAdvertisement { .. } => {
                    match handle_discovery_event(adapter.clone()).await {
                        Ok(devices) => Some(devices),
                        Err(err) => {
                            // On Linux/BlueZ, a peripheral's D-Bus object can be
                            // removed between enumeration and property query when the
                            // device goes out of range. This is a benign race condition
                            // — log at debug level to avoid noise.
                            if is_unknown_object_error(&err) {
                                debug!("Peripheral disappeared before properties could be read (benign race): {err:?}");
                            } else {
                                warn!("Error handling central event: {err:?}");
                            }
                            None
                        }
                    }
                }
                _ => None,
            }
        }
    });

    let initial = iter(vec![initial]);

    let events = select(initial, events);
    Ok(Box::pin(events))
}

async fn handle_discovery_event(adapter: Adapter) -> Result<Vec<DiscoveredDevice>, Error> {
    let peripherals = adapter.peripherals().await?;

    let mut discovered_devices = vec![];

    for peripheral in peripherals {
        let properties = peripheral.properties().await?;
        let id = peripheral.id().to_string();

        let device: DiscoveredDevice = if let Some(properties) = properties {
            (id, properties).into()
        } else {
            DiscoveredDevice {
                id,
                name: None,
                signal_strength: None,
                address: None,
                manufacturer_data: None,
            }
        };

        discovered_devices.push(device);
    }

    discovered_devices.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(discovered_devices)
}

/// Returns `true` when the error is a BlueZ D-Bus `UnknownObject` response.
///
/// This happens on Linux when a peripheral disappears from BlueZ between the
/// time `adapter.peripherals()` enumerates it and when we call
/// `peripheral.properties()`. The D-Bus object path no longer exists, so
/// BlueZ replies with `org.freedesktop.DBus.Error.UnknownObject`. It is a
/// benign race condition and should not be treated as a real error.
fn is_unknown_object_error(err: &Error) -> bool {
    let msg = format!("{err:?}");
    msg.contains("UnknownObject")
}
