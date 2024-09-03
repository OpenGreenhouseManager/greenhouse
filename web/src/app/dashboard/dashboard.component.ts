import { Component } from '@angular/core';
import { NavBarComponent } from '../nav_bar/nav_bar.component';

@Component({
  selector: 'grn-dashboard',
  standalone: true,
  imports: [NavBarComponent],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
})
export class DashboardComponent {}
