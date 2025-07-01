use std::collections::HashMap;

use btleplug::{api::Service, platform::Peripheral as PlatformPeripheral};

use super::discovery::discovered_device::DiscoveredDevice;

#[derive(Debug, Clone)]
pub struct ConnectedDevice {
    pub peripheral: PlatformPeripheral,
    pub device: DiscoveredDevice,
    pub services: HashMap<String, Service>,
    pub client_count: usize,
    /*
    subscriptions: HashMap<Uuid, HashSet<WebsocketClientId>>,
    */
}

impl ConnectedDevice {
    pub fn new(
        peripheral: PlatformPeripheral,
        device: DiscoveredDevice,
        services: HashMap<String, Service>,
    ) -> Self {
        Self {
            peripheral,
            device,
            services,
            client_count: 0,
            /*
            clients: HashSet::new(),
            subscriptions: HashMap::new(),
            */
        }
    }

    pub fn add_client(&mut self) {
        self.client_count += 1;
    }

    pub fn remove_client(&mut self) {
        if self.client_count > 0 {
            self.client_count -= 1;
        }

        /*
        self.subscriptions.values_mut().for_each(|client_ids| {
            client_ids.remove(client_id);
        });
        */
    }

    pub fn has_no_clients(&self) -> bool {
        self.client_count == 0
    }

    /*
    pub fn client_ids(&self) -> Vec<WebsocketClientId> {
        self.clients.iter().cloned().collect()
    }

    pub fn add_subscription(&mut self, client_id: Uuid, characteristic_id: Uuid) -> bool {
        self.subscriptions
            .entry(characteristic_id)
            .or_insert_with(HashSet::new)
            .insert(client_id)
    }

    pub fn subscriptions(
        &mut self,
        characteristic_id: &Uuid,
    ) -> Option<&mut HashSet<WebsocketClientId>> {
        self.subscriptions.get_mut(characteristic_id)
    }

    pub fn remove_subscriptions_for_characteristic(&mut self, uuid: &Uuid) {
        self.subscriptions.remove(uuid);
    }
    */
}
