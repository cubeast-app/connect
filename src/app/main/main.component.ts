import { ChangeDetectionStrategy, Component, ViewChild } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatSlideToggle, MatSlideToggleModule } from '@angular/material/slide-toggle';
import { invoke } from '@tauri-apps/api';
import { getVersion } from '@tauri-apps/api/app';
import { WebviewWindow } from '@tauri-apps/api/window';
import { Observable, from } from 'rxjs';
import { disable, enable, isEnabled } from 'tauri-plugin-autostart-api';

@Component({
  selector: 'app-main',
  templateUrl: './main.component.html',
  styleUrls: ['./main.component.css'],
  imports: [MatSlideToggleModule, MatButtonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  standalone: true
})
export class MainComponent {
  @ViewChild(MatSlideToggle)
  private timerComponent!: MatSlideToggle;

  appVersion!: string;
  discoverWebview: WebviewWindow | undefined;
  helpWebview: WebviewWindow | undefined;

  ngOnInit(): void {
    getVersion().then(version => this.appVersion = version);
    isEnabled().then(enabled => this.timerComponent.checked = enabled);
  }

  ngOnDestroy(): void {
  }

  updateStartOnBoot(start: boolean): void {
    if (start) {
      enable().then(() => { });
    } else {
      disable().then(() => { });
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
        width: 800,
        height: 600,
        center: true
      });

      this.discoverWebview.listen('tauri://error', function (e) {
        console.error(e);
      });
    }

    this.discoverWebview.show();
  }

  help(): void {
    if (this.helpWebview === undefined) {
      this.helpWebview = new WebviewWindow('help', {
        title: 'Help',
        url: '/help',
        width: 800,
        height: 600,
        center: true
      });

      this.helpWebview.listen('tauri://error', function (e) {
        console.error(e);
      });
    }

    this.helpWebview.show();
  }
}
