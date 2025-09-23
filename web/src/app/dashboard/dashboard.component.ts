import { Component } from '@angular/core';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { GraphConfig } from '../shared/graph/graph.component';

@Component({
  selector: 'grn-dashboard',
  standalone: true,
  imports: [NavBarComponent],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
})
export class DashboardComponent {
  protected readonly graphConfig: GraphConfig = {
    device_id: '286cbd49-1aed-463e-a37f-3c2e677ad66d',
  };
}
