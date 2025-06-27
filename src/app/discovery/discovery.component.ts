import { ChangeDetectionStrategy, Component } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { MatSlideToggle } from '@angular/material/slide-toggle';
import { MatTableModule } from '@angular/material/table';
import { PushPipe } from '@ngrx/component';
import { writeText } from '@tauri-apps/api/clipboard';
import { listen } from '@tauri-apps/api/event';
import { BehaviorSubject, combineLatest, distinctUntilChanged, interval, map, Observable, sample } from 'rxjs';
import { DiscoveredDevice } from './discovered_device';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';

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
  imports: [MatTableModule, MatSlideToggle, MatProgressSpinner, MatIcon, PushPipe, MatSnackBarModule ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  standalone: true
})
export class DiscoveryComponent {
  discoveredDevices = new BehaviorSubject<DiscoveredDevice[]>([]);
  shownDevices!: Observable<DiscoveredDevice[]>;
  displayedColumns = ['name', 'address', 'encryption_key'];
  discoveredDevicesFilter = new BehaviorSubject(CubingDeviceFilter);

  constructor(private snackBar: MatSnackBar) { }

  ngOnInit(): void {
    listen('discovery', devices => {
      this.discoveredDevices.next(devices.payload as DiscoveredDevice[]);
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

    this.snackBar.open(`Copied to clipboard`, undefined, {
      duration: 2000,
      horizontalPosition: 'right',
      verticalPosition: 'top'
    });
  }

  trackBy(index: number, device: DiscoveredDevice): string {
    return device.address ?? device.name ?? index.toString();
  }
}

