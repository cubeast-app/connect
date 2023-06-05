import { Component, OnDestroy, OnInit, ViewChild } from "@angular/core";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { invoke } from "@tauri-apps/api/tauri";
import { BehaviorSubject, from, interval, mergeMap, Observable } from "rxjs";
import { disable, enable, isEnabled } from "tauri-plugin-autostart-api";
import { getVersion } from '@tauri-apps/api/app';

const REFRESH_INTERVAL = 1000;

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent implements OnInit, OnDestroy {
  @ViewChild(MatSlideToggle)
  private timerComponent!: MatSlideToggle;
  status!: Observable<number | string>;
  appVersion!: string;
  faCheck = faCheck;
  faTimes = faTimes;

  ngOnInit(): void {
    this.status = interval(REFRESH_INTERVAL).pipe(mergeMap(() => this.refreshStatus()));
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
}
