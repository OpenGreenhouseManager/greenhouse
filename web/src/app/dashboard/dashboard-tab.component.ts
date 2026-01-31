import {
  Component,
  effect,
  inject,
  input,
  model,
  OnDestroy,
  OnInit,
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
  standalone: true,
  imports: [
    GridsterItem,
    Gridster,
    GraphComponent,
    AlertListComponent,
    LatestValueCardComponent,
    CompassCardComponent,
  ],
  templateUrl: './dashboard-tab.component.html',
  //providers: [ConfirmationService],
  styleUrl: './dashboard-tab.component.scss',
})
export class DashboardTabComponent implements OnInit, OnDestroy {
  options: GridsterConfig;
  confirmationService = inject(ConfirmationService);

  dashboard = model.required<DashboardItem[]>();
  editMode = input.required<boolean>();

  private readonly onWindowResize = () => {
    this.options.fixedRowHeight = Math.floor(window.innerHeight / 9);
    this.options['api']?.resize?.();
  };

  constructor() {
    effect(() => {
      if (!this.dashboard()) {
        return;
      }
    });

    this.options = {
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
      fixedRowHeight: Math.floor(window.innerHeight / 9),
      pushResizeItems: true,
      swap: true,
      swapWhileDragging: true,
      minCols: 16,
      minRows: 9,
      maxCols: 16,
      disableScrollHorizontal: true,
      disableScrollVertical: false,
    };
  }

  ngOnInit() {
    window.addEventListener('resize', this.onWindowResize);

    // Load dashboard from backend preferences instead of localStorage
  }

  ngOnDestroy() {
    window.removeEventListener('resize', this.onWindowResize);
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
