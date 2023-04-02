import { Component, OnDestroy, OnInit } from "@angular/core";
import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { invoke } from "@tauri-apps/api/tauri";
import { BehaviorSubject, from, interval, mergeMap, Observable } from "rxjs";

const REFRESH_INTERVAL = 1000;

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent implements OnInit, OnDestroy {
  status!: Observable<number | string>;
  startOnBoot = new BehaviorSubject(false);
  faCheck = faCheck;
  faTimes = faTimes;

  ngOnInit(): void {
    this.status = interval(REFRESH_INTERVAL).pipe(mergeMap(() => this.refreshStatus()));
  }
  
  ngOnDestroy(): void {
  }

  updateStartOnBoot(start: boolean): void {
    this.startOnBoot.next(start);
  }

  refreshStatus(): Observable<number | string> {
    return from(invoke<number | string>("status"));
  }
}
