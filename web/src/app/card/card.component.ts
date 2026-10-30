import { Component, computed, input, output } from '@angular/core';
import { CardModule } from 'primeng/card';

@Component({
  selector: 'grn-card',
  imports: [CardModule],
  templateUrl: './card.component.html',
  styleUrl: './card.component.scss',
})
export class CardComponent {
  public title = input.required<string>();
  public centerTitle = input<boolean>(false);
  public icon = input<string>();
  // Set when nothing listens to iconClicked, so the icon is not a fake button
  public decorativeIcon = input<boolean>(false);
  public icon_class = computed(() => `pi ${this.icon() ?? ''}`);
  public iconClicked = output();
}
