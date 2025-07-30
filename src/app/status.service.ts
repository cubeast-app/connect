import { Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type CheckingForUpdates = {
  type: "checking-for-updates";
}

export type DownloadingUpdate = {
  type: "downloading-update";
  progress: number;
}

export type Running = {
  type: "running";
  version: string;
}

export type AppStatus = CheckingForUpdates | DownloadingUpdate | Running;

@Injectable({
  providedIn: 'root'
})
export class StatusService {
  appStatus = signal<AppStatus>({ type: "checking-for-updates" });

  constructor() {
    this.initializeStatus();
  }

  private async initializeStatus() {
    // Get initial status
    try {
      const status = await invoke<AppStatus>("app_status");
      this.appStatus.set(status);
    } catch (error) {
      console.error('Failed to get initial app status:', error);
    }

    // Listen for status updates
    listen<AppStatus>('app_status_changed', (event) => {
      this.appStatus.set(event.payload);
    });
  }

  getStatusText(): string {
    const status = this.appStatus();
    switch (status.type) {
      case 'checking-for-updates':
        return 'Checking for updates...';
      case 'downloading-update':
        return `Downloading update... ${status.progress}%`;
      case 'running':
        return `v${status.version}`;
      default:
        return 'Unknown status';
    }
  }
}
