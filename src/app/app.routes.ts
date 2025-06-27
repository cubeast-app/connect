import { Routes } from "@angular/router";
import { DiscoveryComponent } from "./discovery/discovery.component";
import { MainComponent } from "./main/main.component";

export const routes: Routes = [
  { path: 'discovery', component: DiscoveryComponent },
  { path: '', component: MainComponent },
];
