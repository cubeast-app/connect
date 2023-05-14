use std::collections::{HashMap, HashSet};

use btleplug::platform::Peripheral as PlatformPeripheral;
use uuid::Uuid;

use crate::discovered_device::DiscoveredDevice;

pub type ClientId = Uuid;
pub type DeviceId = String;

pub struct ConnectedDevice {
    clients: HashSet<ClientId>,
    pub peripheral: PlatformPeripheral,
    pub device: DiscoveredDevice,
    pub services: Vec<Uuid>,
    subscriptions: HashMap<Uuid, HashSet<ClientId>>,
}

impl ConnectedDevice {
    pub fn new(
        peripheral: PlatformPeripheral,
        device: DiscoveredDevice,
        services: Vec<Uuid>,
    ) -> Self {
        Self {
            clients: HashSet::new(),
            peripheral,
            device,
            services,
            subscriptions: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client_id: Uuid) {
        self.clients.insert(client_id);
    }

    pub fn remove_client(&mut self, client_id: &Uuid) {
        self.clients.remove(client_id);

        self.subscriptions.values_mut().for_each(|client_ids| {
            client_ids.remove(client_id);
        });
    }

    pub fn has_no_clients(&self) -> bool {
        self.clients.is_empty()
    }

    pub fn client_ids(&self) -> Vec<ClientId> {
        self.clients.iter().cloned().collect()
    }

    pub fn add_subscription(&mut self, client_id: Uuid, characteristic_id: Uuid) -> bool {
        self.subscriptions
            .entry(characteristic_id)
            .or_insert_with(HashSet::new)
            .insert(client_id)
    }

    pub fn subscriptions(&mut self, characteristic_id: &Uuid) -> Option<&mut HashSet<ClientId>> {
        self.subscriptions.get_mut(characteristic_id)
    }

    pub fn remove_subscriptions_for_characteristic(&mut self, uuid: &Uuid) {
        self.subscriptions.remove(uuid);
    }
}
