import { Component } from '@angular/core';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'grn-nav-bar',
  standalone: true,
  imports: [RouterModule],
  templateUrl: './nav_bar.component.html',
  styleUrl: './nav_bar.component.scss',
})
export class NavBarComponent {
  constructor() {}
}
