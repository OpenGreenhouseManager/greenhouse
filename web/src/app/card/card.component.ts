import { Component, input } from '@angular/core';
import { PanelModule } from 'primeng/panel';

@Component({
  selector: 'grn-card',
  standalone: true,
  imports: [PanelModule],
  templateUrl: './card.component.html',
  styleUrl: './card.component.scss',
})
export class CardComponent {}
