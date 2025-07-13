import { ChangeDetectionStrategy, Component } from '@angular/core';
import { MatExpansionModule } from '@angular/material/expansion';

@Component({
  selector: 'app-help',
  imports: [MatExpansionModule],
  templateUrl: './help.component.html',
  styleUrl: './help.component.css',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class HelpComponent {

}
