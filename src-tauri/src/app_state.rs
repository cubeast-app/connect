use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{
    clients::Clients,
    connected_device::{ClientId, ConnectedDevice, DeviceId},
};

pub struct AppState<ClientType: Clone> {
    clients: Clients<ClientType>,
    discovery_clients: HashSet<ClientId>,
    connected_devices: HashMap<DeviceId, ConnectedDevice>,
}

impl<ClientType: Clone> AppState<ClientType> {
    pub fn new() -> Self {
        Self {
            clients: Clients::new(),
            discovery_clients: HashSet::new(),
            connected_devices: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client: ClientType) -> Uuid {
        self.clients.add(client)
    }

    pub fn remove_client(&mut self, client_id: &Uuid) {
        self.clients.remove(client_id);

        // Remove the client from the discovery clients
        self.discovery_clients.remove(client_id);

        // Remove the client from all connected devices
        // This includes unsubscribing from notifications
        self.connected_devices
            .values_mut()
            .for_each(|device| device.remove_client(client_id));
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn client_ids(&self) -> Vec<ClientId> {
        self.clients.client_ids()
    }

    pub fn client(&self, client_id: &Uuid) -> Option<ClientType> {
        self.clients.get_by_id(client_id)
    }

    pub fn add_discovery_client(&mut self, client_id: Uuid) {
        self.discovery_clients.insert(client_id);
    }

    pub fn remove_discovery_client(&mut self, client_id: &Uuid) {
        self.discovery_clients.remove(client_id);
    }

    pub fn has_discovery_clients(&self) -> bool {
        !self.discovery_clients.is_empty()
    }

    pub fn discovery_client_ids(&self) -> Vec<Uuid> {
        self.discovery_clients.iter().cloned().collect()
    }

    pub fn discovery_clients(&self) -> Vec<ClientType> {
        self.discovery_clients
            .iter()
            .filter_map(|id| self.client(id))
            .collect()
    }

    pub fn connected_device(&mut self, device_id: &DeviceId) -> Option<&mut ConnectedDevice> {
        self.connected_devices.get_mut(device_id)
    }

    pub fn connected_devices_with_no_clients(&self) -> Vec<&ConnectedDevice> {
        self.connected_devices
            .values()
            .filter(|connected_device| connected_device.has_no_clients())
            .collect()
    }

    pub fn add_connected_device(
        &mut self,
        connected_device: ConnectedDevice,
    ) -> &mut ConnectedDevice {
        self.connected_devices
            .entry(connected_device.device.id.clone())
            .or_insert(connected_device)
    }

    pub fn device_disconnected(&mut self, device_id: &str) {
        self.connected_devices.remove(device_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lets_expect::lets_expect;

    lets_expect! {
        let empty_state = AppState::new();
        let state_with_one_client = {
            let mut s = AppState::new();
            s.add_client(alice.clone());
            s
        };
        let state_with_two_clients = {
            let mut s = AppState::new();
            s.add_client(alice.clone());
            s.add_client(bob);
            s
        };
        let state_with_one_discovery_client = {
            let mut s = AppState::new();
            let alice_id = s.add_client(alice.clone());
            s.add_client(bob);
            s.add_discovery_client(alice_id);
            s
        };
        let alice = "alice".to_string();
        let bob = "bob".to_string();

        expect(state.add_client(alice.clone())) as add_client {
            when(mut state = empty_state) to make(state.client_count()) equal(1)
            when(mut state = state_with_one_client) to make(state.client_count()) equal(2)
        }

        expect(state.remove_client(&client_id)) as remove_client {
            when(mut state = state_with_one_client) {
                let client_id = *state.client_ids().first().unwrap();

                to make(state.client_count()) equal(0)
            }

            when(mut state = state_with_two_clients) {
                let client_id = *state.client_ids().first().unwrap();

                to make(state.client_count()) equal(1)
            }

            when(mut state = state_with_one_discovery_client) {
                let client_id = *state.discovery_client_ids().first().unwrap();

                to make(state.has_discovery_clients()) equal(false)
            }
        }

        expect(state.add_discovery_client(client_id)) as add_discovery_client {
            when(mut state = state_with_one_client) {
                let client_id = *state.client_ids().first().unwrap();
                to make(state.has_discovery_clients()) be_true
            }
        }

        expect(state.remove_discovery_client(&client_id)) as remove_discovery_client {
            when(mut state = state_with_one_discovery_client) {
                let client_id = *state.discovery_client_ids().first().unwrap();
                to make(state.has_discovery_clients()) be_false
            }
        }
    }
}
