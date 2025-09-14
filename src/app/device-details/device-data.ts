export interface CharacteristicData {
  uuid: string;
  read: boolean;
  write: boolean;
  write_without_response: boolean;
  notify: boolean;
}

export interface ServiceData {
  uuid: string;
  characteristics: CharacteristicData[];
}

export interface DeviceData {
  id: string;
  name: string;
  address: string;
  manufacturer_data: Record<string, any>;
  services: ServiceData[];
}
