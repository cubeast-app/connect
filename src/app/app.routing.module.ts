import { NgModule } from '@angular/core';
import { Routes, RouterModule } from '@angular/router';
import { DiscoveryComponent } from './discovery/discovery.component';
import { MainComponent } from './main/main.component';

const routes: Routes = [
    { path: 'discovery', component: DiscoveryComponent },
    { path: '', component: MainComponent },
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }