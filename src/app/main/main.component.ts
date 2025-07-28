import { ChangeDetectionStrategy, Component, signal, ViewChild } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatSlideToggle, MatSlideToggleModule } from '@angular/material/slide-toggle';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { Router } from '@angular/router';
import { Observable, from } from 'rxjs';
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';

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
  private autostartToggle!: MatSlideToggle;

  appVersion = signal<string | undefined>(undefined);

  constructor(private router: Router) {}

  ngOnInit(): void {
    getVersion().then(version => this.appVersion.set(version));
    isEnabled().then(enabled => this.autostartToggle.checked = enabled);
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
    this.router.navigate(['/discovery']);
  }

  help(): void {
    this.router.navigate(['/help']);
  }
}
