import { Component, ViewChild } from '@angular/core';
import { MatSlideToggle } from '@angular/material/slide-toggle';
import { faCheck, faTimes } from '@fortawesome/free-solid-svg-icons';
import { invoke } from '@tauri-apps/api';
import { getVersion } from '@tauri-apps/api/app';
import { TauriEvent, UnlistenFn, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/window';
import { Observable, interval, mergeMap, from } from 'rxjs';
import { isEnabled, enable, disable } from 'tauri-plugin-autostart-api';

@Component({
  selector: 'app-main',
  templateUrl: './main.component.html',
  styleUrls: ['./main.component.css']
})
export class MainComponent {
  @ViewChild(MatSlideToggle)
  private timerComponent!: MatSlideToggle;
  appVersion!: string;
  faCheck = faCheck;
  faTimes = faTimes;
  discoverWebview: WebviewWindow | undefined;

  ngOnInit(): void {
    getVersion().then(version => this.appVersion = version);
    isEnabled().then(enabled => this.timerComponent.checked = enabled);
  }
  
  ngOnDestroy(): void {
  }

  updateStartOnBoot(start: boolean): void {
    if (start) {
      enable().then(() => {});
    } else {
      disable().then(() => {});
    }
  }

  refreshStatus(): Observable<number | string> {
    return from(invoke<number | string>("status"));
  }

  discover(): void {
    if (this.discoverWebview === undefined) {
      this.discoverWebview = new WebviewWindow('discovery', {
        title: 'Discovery',
        url: '/discovery',
        width: 600,
        height: 400,
      });

      this.discoverWebview.listen('tauri://error', function (e) {
        console.error(e);
      });

      this.discoverWebview.listen(TauriEvent.WINDOW_CLOSE_REQUESTED, () => {
        this.stopDiscovery().catch(console.error);
      });
    }

    this.discoverWebview.show();

    this.startDiscovery().catch(console.error);
  }

  private async startDiscovery(): Promise<void> {
    return invoke("start_discovery");
  }

  private async stopDiscovery(): Promise<void> {
    return invoke("stop_discovery");
  }
}
