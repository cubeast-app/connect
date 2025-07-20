import { Component, OnInit } from '@angular/core';
import { listen } from '@tauri-apps/api/event';
import { DeviceData } from './device-data';
import { CommonModule } from '@angular/common';
import { invoke } from '@tauri-apps/api';
import { ActivatedRoute } from '@angular/router';
import { BehaviorSubject, first } from 'rxjs';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { writeText } from '@tauri-apps/api/clipboard';
import { LetDirective, PushPipe } from '@ngrx/component';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-device-details',
  templateUrl: './device-details.component.html',
  styleUrls: ['./device-details.component.css'],
  imports: [CommonModule, MatProgressSpinnerModule, LetDirective, MatSnackBarModule, MatButtonModule],
  standalone: true
})
export class DeviceDetailsComponent implements OnInit {
  deviceDetails: BehaviorSubject<DeviceData | null | undefined> = new BehaviorSubject<DeviceData | null | undefined>(undefined);

  constructor(private route: ActivatedRoute, private snackBar: MatSnackBar) { }

  ngOnInit(): void {
    console.log('DeviceDetailsComponent initialized');

    this.tryAgain();
  }

  tryAgain(): void {
    this.deviceDetails.next(undefined);
    const deviceName = this.route.snapshot.paramMap.get('device_name');
    if (deviceName) {
      invoke<DeviceData>('device_details', { deviceName }).then(data => {
        this.deviceDetails.next(data);
      }).catch(error => {
        console.error('Error fetching device details:', error);
        this.deviceDetails.next(null);
      });
    }
  }

  copyToClipboard(): void {
    this.deviceDetails.pipe(first()).subscribe(device => {
      if (device) {
        const deviceInfo = JSON.stringify(device, null, 2);
        writeText(deviceInfo).then(() => {
          this.snackBar.open(`Copied to clipboard`, undefined, {
            duration: 2000,
            horizontalPosition: 'right',
            verticalPosition: 'top'
          });
          console.log('Device details copied to clipboard');
        }).catch(err => {
          console.error('Failed to copy device details:', err);
        });
      }
    });
  }
}
