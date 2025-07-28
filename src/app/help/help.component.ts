import { ChangeDetectionStrategy, Component } from '@angular/core';
import { MatExpansionModule } from '@angular/material/expansion';
import { MatButtonModule } from '@angular/material/button';
import { Router } from '@angular/router';

@Component({
  selector: 'app-help',
  imports: [MatExpansionModule, MatButtonModule],
  templateUrl: './help.component.html',
  styleUrl: './help.component.css',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class HelpComponent {
  constructor(private router: Router) {}

  goBack(): void {
    this.router.navigate(['/']);
  }
}
