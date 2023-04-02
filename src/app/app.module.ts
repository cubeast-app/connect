import { NgModule } from "@angular/core";
import { BrowserModule } from "@angular/platform-browser";
import { LetModule, PushModule } from "@ngrx/component";
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';


import { AppComponent } from "./app.component";
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';

@NgModule({
  declarations: [AppComponent],
  imports: [BrowserModule, PushModule, LetModule, BrowserAnimationsModule, MatCheckboxModule, MatSlideToggleModule, MatProgressSpinnerModule, FontAwesomeModule],
  providers: [],
  bootstrap: [AppComponent],
})
export class AppModule {}
