import { Component, OnInit } from '@angular/core';
import { DeviceData } from './device-data';
import { CommonModule } from '@angular/common';
import { invoke } from '@tauri-apps/api/core';
import { ActivatedRoute, Router } from '@angular/router';
import { BehaviorSubject } from 'rxjs';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { LetDirective } from '@ngrx/component';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatButtonModule } from '@angular/material/button';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

@Component({
  selector: 'app-device-details',
  templateUrl: './device-details.component.html',
  styleUrls: ['./device-details.component.css'],
  imports: [CommonModule, MatProgressSpinnerModule, LetDirective, MatSnackBarModule, MatButtonModule],
  standalone: true
})
export class DeviceDetailsComponent implements OnInit {
  deviceDetails: BehaviorSubject<DeviceData | null | undefined> = new BehaviorSubject<DeviceData | null | undefined>(undefined);

  constructor(private route: ActivatedRoute, private router: Router, private snackBar: MatSnackBar) { }

  ngOnInit(): void {
    this.tryAgain();
  }

  tryAgain(): void {
    this.deviceDetails.next(undefined);
    const deviceId = this.route.snapshot.paramMap.get('device_id');
    if (deviceId) {
      invoke<DeviceData>('device_details', { deviceId }).then(data => {
        this.deviceDetails.next(data);
      }).catch(error => {
        console.error('Error fetching device details:', error);
        this.deviceDetails.next(null);
      });
    }
  }

  copyToClipboard(): void {
    const deviceInfo = JSON.stringify(this.deviceDetails.value, null, 2);

    writeText(deviceInfo).then(() => {
      this.snackBar.open(`Copied to clipboard`, undefined, {
        duration: 2000,
        horizontalPosition: 'right',
        verticalPosition: 'top'
      });
    }).catch(err => {
      console.error('Failed to copy device details:', err);
    });
  }

  goBack(): void {
    this.router.navigate(['/discovery']);
  }
}
