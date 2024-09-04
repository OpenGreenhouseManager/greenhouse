import { CommonModule } from '@angular/common';
import { Component, computed, input, output } from '@angular/core';
import { CardModule } from 'primeng/card';

@Component({
  selector: 'grn-card',
  standalone: true,
  imports: [CommonModule, CardModule],
  templateUrl: './card.component.html',
  styleUrl: './card.component.scss',
})
export class CardComponent {
  public title = input.required<string>();
  public centerTitle = input<boolean>(false);
  public icon = input<string>();
  public icon_class = computed(() => `pi ${this.icon() ?? ''}`);
  public iconClicked = output();
}
