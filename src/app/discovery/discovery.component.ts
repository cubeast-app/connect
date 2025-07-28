import { ChangeDetectionStrategy, Component, signal } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { MatSlideToggle } from '@angular/material/slide-toggle';
import { MatTableModule } from '@angular/material/table';
import { LetDirective } from '@ngrx/component';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { listen } from '@tauri-apps/api/event';
import { BehaviorSubject, combineLatest, distinctUntilChanged, from, interval, map, merge, Observable, of, sample } from 'rxjs';
import { DiscoveredDevice } from './discovered-device';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { Router } from '@angular/router';
import { invoke } from '@tauri-apps/api/core';
import { MatButtonModule } from '@angular/material/button';
import { CommonModule } from '@angular/common';

type DiscoveredDevicesFilter = (devices: DiscoveredDevice[]) => DiscoveredDevice[];

const NoFilter: DiscoveredDevicesFilter = devices => devices;
const CubingPrefixes = ['GAN', 'MG', 'AiCube', 'Gi', 'Mi Smart Magic Cube', 'GoCube', 'Rubiks', 'MHC', 'WCU'];
const CubingDeviceFilter: DiscoveredDevicesFilter = devices => devices.filter(isCubingDevice);
const ScanTimeout = 30000;

function isCubingDevice(device: DiscoveredDevice): boolean {
  return CubingPrefixes.some(prefix => device.name?.startsWith(prefix));
}

@Component({
  selector: 'app-discovery',
  templateUrl: './discovery.component.html',
  styleUrls: ['./discovery.component.css'],
  imports: [MatTableModule, MatSlideToggle, MatProgressSpinner, MatIcon, MatSnackBarModule, LetDirective, MatButtonModule, CommonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  standalone: true
})
export class DiscoveryComponent {
  discoveredDevices = new BehaviorSubject<DiscoveredDevice[]>([]);
  shownDevices!: Observable<DiscoveredDevice[]>;
  displayedColumns = ['name', 'address', 'encryption_key', 'actions'];
  discoveredDevicesFilter = new BehaviorSubject(CubingDeviceFilter);
  isScanning = new BehaviorSubject<boolean>(true);

  constructor(private snackBar: MatSnackBar, private router: Router) { }

  ngOnInit(): void {
    listen('discovery', devices => {
      this.discoveredDevices.next(devices.payload as DiscoveredDevice[]);
    });

    // only emit values if they are distinct from the previous value, use a deep comparison of the array elements
    const distinctDiscoveredDevices = this.discoveredDevices.pipe(distinctUntilChanged((a, b) => a.map(device => device.id).join(',') === b.map(device => device.id).join(',')));
    // emit new array at most once per second
    const throttled = distinctDiscoveredDevices.pipe(sample(interval(1000).pipe(startWith(0))));

    this.shownDevices = combineLatest([throttled, this.discoveredDevicesFilter]).pipe(map(([devices, filter]) => {
      const namedOrAddressed = devices.filter(device => device.name !== undefined || device.address !== undefined);
      const filtered = filter(namedOrAddressed);

      // sort by name, or if names are equal, unnamed devices should be after named devices, use address or id if empty name
      return filtered.sort((a, b) => {
        if (a.name && b.name) {
          return a.name!.localeCompare(b.name!);
        } else if (a.name && !b.name) {
          return -1;
        }
        else if (!a.name && b.name) {
          return 1;
        }

        return a.id.localeCompare(b.id);
      });
    }));

    this.reScan();
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

  details(device: DiscoveredDevice): void {
    this.router.navigate(['/device-details', device.id]);
  }

  goBack(): void {
    this.router.navigate(['/']);
  }

  trackBy(index: number, device: DiscoveredDevice): string {
    return device.id;
  }

  reScan(): void {
    this.discoveredDevices.next([]);

    this.startDiscovery().catch(() => {
      this.snackBar.open('Failed to start discovery', undefined, {
        duration: 2000,
        horizontalPosition: 'right',
        verticalPosition: 'top'
      });
    });
  }

  private async startDiscovery(): Promise<void> {
    await invoke("start_discovery");

    this.isScanning.next(true);

    setTimeout(() => {
      this.stopDiscovery().catch(console.error);
    }, ScanTimeout);
  }

  private async stopDiscovery(): Promise<void> {
    this.isScanning.next(false);
    return invoke("stop_discovery");
  }
}

