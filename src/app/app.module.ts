import { NgModule } from "@angular/core";
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatTableModule } from '@angular/material/table';
import { BrowserModule } from "@angular/platform-browser";
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { LetDirective, PushPipe } from "@ngrx/component";
import { AppComponent } from "./app.component";
import { AppRoutingModule } from "./app.routing.module";
import { DiscoveryComponent } from './discovery/discovery.component';
import { MainComponent } from './main/main.component';
import { MatIconModule } from '@angular/material/icon';


@NgModule({
  declarations: [
    AppComponent,
    DiscoveryComponent,
    MainComponent
  ],
  imports: [
    AppRoutingModule,
    BrowserModule,
    PushPipe,
    LetDirective,
    BrowserAnimationsModule,
    MatSlideToggleModule,
    MatProgressSpinnerModule,
    MatButtonModule,
    MatTableModule,
    MatIconModule
  ],
  providers: [],
  bootstrap: [AppComponent],
})
export class AppModule { }
