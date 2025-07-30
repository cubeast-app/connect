import { ChangeDetectionStrategy, Component, signal, ViewChild } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatSlideToggle, MatSlideToggleModule } from '@angular/material/slide-toggle';
import { invoke } from '@tauri-apps/api/core';
import { Router } from '@angular/router';
import { Observable, from } from 'rxjs';
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { relaunch } from '@tauri-apps/plugin-process';
import { openUrl } from '@tauri-apps/plugin-opener';
import { StatusService } from '../status.service';

@Component({
  selector: 'app-main',
  templateUrl: './main.component.html',
  styleUrls: ['./main.component.scss'],
  imports: [MatSlideToggleModule, MatButtonModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
  standalone: true
})
export class MainComponent {
  @ViewChild(MatSlideToggle)
  private autostartToggle!: MatSlideToggle;

  constructor(private router: Router, public statusService: StatusService) { }

  ngOnInit(): void {
    isEnabled().then(enabled => this.autostartToggle.checked = enabled);
  }

  ngOnDestroy(): void {
  }

  startOnBoot(start: boolean): void {
    if (start) {
      enable().then(() => { });
    } else {
      disable().then(() => { });
    }
  }

  discover(): void {
    this.router.navigate(['/discovery']);
  }

  help(): void {
    this.router.navigate(['/help']);
  }

  openCubeast(): void {
    openUrl('https://cubeast.com');
  }

  async update(): Promise<void> {
    await relaunch();
  }
}

