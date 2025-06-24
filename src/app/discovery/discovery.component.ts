import { ChangeDetectionStrategy, Component, NgZone } from '@angular/core';
import { DiscoveredDevice, ManufacturerData } from './discovered_device';
import { BehaviorSubject, Observable, Subject, combineLatest, distinctUntilChanged, interval, map, sample } from 'rxjs';
import { listen } from '@tauri-apps/api/event'
import { writeText } from '@tauri-apps/api/clipboard';


type DiscoveredDevicesFilter = (devices: DiscoveredDevice[]) => DiscoveredDevice[];

const NoFilter: DiscoveredDevicesFilter = devices => devices;
const CubingPrefixes = ['GAN', 'MG', 'AiCube', 'Gi', 'Mi Smart Magic Cube', 'GoCube', 'Rubiks', 'MHC', 'WCU'];
const DefaultName = 'Unnamed Device';
const CubingDeviceFilter: DiscoveredDevicesFilter = devices => devices.filter(isCubingDevice);

function isCubingDevice(device: DiscoveredDevice): boolean {
  return CubingPrefixes.some(prefix => device.name?.startsWith(prefix));
}

@Component({
    selector: 'app-discovery',
    templateUrl: './discovery.component.html',
    styleUrls: ['./discovery.component.css'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    standalone: false
})
export class DiscoveryComponent {
  discoveredDevices = new BehaviorSubject<DiscoveredDevice[]>([]);
  shownDevices!: Observable<DiscoveredDevice[]>;
  columnsToDisplay = ['name', 'address', 'encryption_key'];
  discoveredDevicesFilter = new BehaviorSubject<DiscoveredDevicesFilter>(CubingDeviceFilter);

  constructor(private zone: NgZone) { }

  ngOnInit(): void {
    listen('discovery', devices => {
      this.zone.run(() => {
          this.discoveredDevices.next(devices.payload as DiscoveredDevice[]);
      });
    });

    // only emit values if they are distinct from the previous value, use a deep comparison of the array elements
    const distinctDiscoveredDevices = this.discoveredDevices.pipe(distinctUntilChanged((a, b) => JSON.stringify(a) === JSON.stringify(b)));
    // emit new array at most once per second
    const throttled = distinctDiscoveredDevices.pipe(sample(interval(1000)));

    this.shownDevices = combineLatest([throttled, this.discoveredDevicesFilter]).pipe(map(([devices, filter]) => {
      const namedOrAddressed = devices.filter(device => device.name !== undefined || device.address !== undefined);
      const filtered = filter(namedOrAddressed);

      // if devices have no name, set it to DefaultName
      const named = filtered.map(device => {
        device.name = device.name ?? DefaultName;
        return device;
      });

      // sort by name, or if names are equal, by address if address is defined
      return named.sort((a, b) => {
        if (a.name === b.name) {
          return a.address === undefined ? 1 : b.address === undefined ? -1 : a.address.localeCompare(b.address);
        } else {
          return a.name!.localeCompare(b.name!);
        }
      });
    }));
  }

  showOnlyCubingDevices(showOnlyCubingDevices: boolean): void {
    if (showOnlyCubingDevices) {
      this.discoveredDevicesFilter.next(CubingDeviceFilter);
    } else {
      this.discoveredDevicesFilter.next(NoFilter);
    }
  }

  encryptionKey(device: DiscoveredDevice): string | undefined {
    if (!isCubingDevice(device) || device.manufacturer_data === undefined) {
      return undefined;
    }

    const indexes = Object.keys(device.manufacturer_data).map(Number);
    const manufacturerDataPart = device.manufacturer_data[indexes[0]];

    if (manufacturerDataPart === undefined) {
      return undefined;
    }

    // extract the last 6 bytes of the manufacturer data, reverse them and format them as a MAC address
    return manufacturerDataPart.slice(-6).reverse().map(byte => byte.toString(16).padStart(2, '0')).join(':').toUpperCase();
  }

  async copyToClipboard(text: string): Promise<void> {
    await writeText(text);
  }

  trackBy(index: number, device: DiscoveredDevice): string {
    return device.address ?? device.name ?? index.toString();
  }
}

