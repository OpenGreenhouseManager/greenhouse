import {
  Component,
  computed,
  inject,
  input,
  model,
  signal,
} from '@angular/core';
import {
  Gridster,
  GridsterConfig,
  GridsterItem,
  GridsterItemConfig,
  GridType,
} from 'angular-gridster2';
import { ConfirmationService } from 'primeng/api';
import { AlertListComponent } from '../shared/alert/alert-list.component';
import { CompassCardComponent } from '../shared/compass/compass-card.component';
import { GraphComponent, GraphConfig } from '../shared/graph/graph.component';
import { LatestValueCardComponent } from '../shared/value/latest-value-card.component';

export interface GraphDashboardItem extends GridsterItemConfig {
  type: 'graph';
  options: GraphConfig & { name: string };
}

export interface MultiGraphDashboardItem extends GridsterItemConfig {
  type: 'multiGraph';
  options: GraphConfig & { name: string };
}

export interface AlertListDashboardItem extends GridsterItemConfig {
  type: 'alertList';
  options: {
    dataSourceId: string;
    name: string;
  };
}

export interface LatestValueDashboardItem extends GridsterItemConfig {
  type: 'latestValue';
  options: {
    deviceId: string;
    subProperty?: string | undefined;
    name: string;
  };
}

export interface CompassDashboardItem extends GridsterItemConfig {
  type: 'compass';
  options: {
    deviceId: string;
    subProperty?: string | undefined;
    name: string;
  };
}

export type DashboardItem =
  | GraphDashboardItem
  | MultiGraphDashboardItem
  | AlertListDashboardItem
  | LatestValueDashboardItem
  | CompassDashboardItem;

@Component({
  selector: 'grn-dashboard-tab',
  imports: [
    GridsterItem,
    Gridster,
    GraphComponent,
    AlertListComponent,
    LatestValueCardComponent,
    CompassCardComponent,
  ],
  templateUrl: './dashboard-tab.component.html',
  styleUrl: './dashboard-tab.component.scss',
  host: {
    '(window:resize)': 'onWindowResize()',
  },
})
export class DashboardTabComponent {
  confirmationService = inject(ConfirmationService);

  dashboard = model.required<DashboardItem[]>();
  editMode = input.required<boolean>();

  private rowHeight = signal(Math.floor(window.innerHeight / 9));

  // Gridster's `options` is a signal input, so a resize must hand it a new
  // object; mutating the old one is not picked up.
  options = computed<GridsterConfig>(() => ({
    gridType: GridType.ScrollVertical,
    draggable: {
      enabled: true,
      stop: () => {
        //trigger change detection
        this.dashboard.update(items => [...items]);
      },
    },
    resizable: {
      enabled: true,
      stop: () => {
        //trigger change detection
        this.dashboard.update(items => [...items]);
      },
    },
    fixedRowHeight: this.rowHeight(),
    pushResizeItems: true,
    swap: true,
    swapWhileDragging: true,
    minCols: 16,
    minRows: 9,
    maxCols: 16,
    disableScrollHorizontal: true,
    disableScrollVertical: false,
  }));

  onWindowResize() {
    this.rowHeight.set(Math.floor(window.innerHeight / 9));
  }

  removeItem($event: MouseEvent | TouchEvent, item: DashboardItem): void {
    $event.preventDefault();
    $event.stopPropagation();
    this.confirmationService.confirm({
      header: 'Remove Item',
      message: 'Are you sure you want to remove this item?',
      accept: () => {
        this.dashboard.update(items => items.filter(i => i !== item));
      },
    });
  }
}
