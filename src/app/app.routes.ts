import { Routes } from "@angular/router";
import { DiscoveryComponent } from "./discovery/discovery.component";
import { MainComponent } from "./main/main.component";
import { HelpComponent } from "./help/help.component";
import { DeviceDetailsComponent } from "./device-details/device-details.component";

export const routes: Routes = [
  { path: 'device-details/:device_id', component: DeviceDetailsComponent},
  { path: 'discovery', component: DiscoveryComponent },
  { path: 'help', component: HelpComponent },
  { path: '', component: MainComponent },
];
