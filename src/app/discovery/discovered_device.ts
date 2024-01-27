export type ManufacturerData = {[index: number]: number[]};

export interface DiscoveredDevice {
    id: string,
    name?: string,
    address?: string,
    signal_strength?: number,
    manufacturer_data?: ManufacturerData,
}