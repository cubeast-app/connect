import { Routes } from "@angular/router";
import { DiscoveryComponent } from "./discovery/discovery.component";
import { MainComponent } from "./main/main.component";
import { HelpComponent } from "./help/help.component";

export const routes: Routes = [
  { path: 'discovery', component: DiscoveryComponent },
  { path: 'help', component: HelpComponent },
  { path: '', component: MainComponent },
];
