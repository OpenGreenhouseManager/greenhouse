import type { OnDestroy, OnInit } from '@angular/core';
import { Component, inject, signal } from '@angular/core';
import type { GridsterConfig, GridsterItemConfig } from 'angular-gridster2';
import { Gridster, GridsterItem, GridType } from 'angular-gridster2';
import { ConfirmationService } from 'primeng/api';
import { ButtonModule } from 'primeng/button';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { UserPreferencesService } from '../services/user-preferences.service';
import { AlertListComponent } from '../shared/alert/alert-list.component';
import { CompassCardComponent } from '../shared/compass/compass-card.component';
import type { GraphConfig } from '../shared/graph/graph.component';
import { GraphComponent } from '../shared/graph/graph.component';
import { LatestValueCardComponent } from '../shared/value/latest-value-card.component';
import type { DashboardComponentCreate } from './add-dashboard-component-dialog.component';
import { AddDashboardComponentDialogComponent } from './add-dashboard-component-dialog.component';

interface GraphDashboardItem extends GridsterItemConfig {
  type: 'graph';
  options: GraphConfig & { name: string };
}

interface MultiGraphDashboardItem extends GridsterItemConfig {
  type: 'multiGraph';
  options: GraphConfig & { name: string };
}

interface AlertListDashboardItem extends GridsterItemConfig {
  type: 'alertList';
  options: {
    dataSourceId: string;
    name: string;
  };
}

interface LatestValueDashboardItem extends GridsterItemConfig {
  type: 'latestValue';
  options: {
    deviceId: string;
    subProperty?: string | undefined;
    name: string;
  };
}

interface CompassDashboardItem extends GridsterItemConfig {
  type: 'compass';
  options: {
    deviceId: string;
    subProperty?: string | undefined;
    name: string;
  };
}

// Union type for all dashboard items
type DashboardItem =
  | GraphDashboardItem
  | MultiGraphDashboardItem
  | AlertListDashboardItem
  | LatestValueDashboardItem
  | CompassDashboardItem;

@Component({
  selector: 'grn-dashboard',
  standalone: true,
  imports: [
    NavBarComponent,
    GridsterItem,
    Gridster,
    GraphComponent,
    AlertListComponent,
    LatestValueCardComponent,
    CompassCardComponent,
    ButtonModule,
    AddDashboardComponentDialogComponent,
    ConfirmDialogModule,
  ],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
  providers: [ConfirmationService],
})
export class DashboardComponent implements OnDestroy, OnInit {
  options: GridsterConfig;
  dashboard = signal<DashboardItem[]>([]);
  dialogVisible = signal(false);
  editMode = signal(false);
  private confirmationService = inject(ConfirmationService);
  private userPreferencesService = inject(UserPreferencesService);
  private readonly onWindowResize = () => {
    this.options.fixedRowHeight = Math.floor(window.innerHeight / 9);
    this.options['api']?.resize?.();
  };

  constructor() {
    this.options = {
      gridType: GridType.ScrollVertical,
      draggable: {
        enabled: true,
        stop: this.saveDashboard.bind(this), // called after drag
      },
      resizable: {
        enabled: true,
        stop: this.saveDashboard.bind(this), // called after resize
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
    this.loadDashboard();
  }

  private loadDashboard() {
    this.userPreferencesService.getUserPreferences().subscribe({
      next: preferences => {
        if (preferences.dashboard_preferences) {
          this.dashboard.set(JSON.parse(preferences.dashboard_preferences));
        } else {
          this.dashboard.set([]);
        }
      },
      error: () => {
        this.dashboard.set([]);
      },
    });
  }

  ngOnDestroy() {
    window.removeEventListener('resize', this.onWindowResize);
  }

  saveDashboard(_: unknown) {
    setTimeout(() => {
      // Save to backend instead of localStorage
      this.userPreferencesService.getUserPreferences().subscribe({
        next: currentPreferences => {
          const updatedPreferences = {
            ...currentPreferences,
            dashboard_preferences: JSON.stringify(this.dashboard()),
          };

          this.userPreferencesService
            .updateUserPreferences(updatedPreferences)
            .subscribe({
              next: () => {},
              error: () => {},
            });
        },
        error: () => {},
      });
    }, 300);
  }

  addEntry() {
    this.dialogVisible.set(true);
  }

  onDialogClose() {
    this.dialogVisible.set(false);
  }

  onComponentAdded(componentData: DashboardComponentCreate) {
    // Create the appropriate dashboard item based on type
    let newItem: DashboardItem;

    if (componentData.type === 'graph') {
      const data = componentData.graphData?.[0];
      if (!data) {
        return;
      }
      newItem = {
        cols: 3,
        rows: 3,
        y: 0,
        x: 0,
        type: 'graph',
        options: {
          graph_data: [
            {
              device_id: data.deviceId,
              sub_property: data.subProperty,
            },
          ],
          name: componentData.name,
        },
      } satisfies GraphDashboardItem;
    } else if (componentData.type === 'multiGraph') {
      newItem = {
        cols: 4,
        rows: 3,
        y: 0,
        x: 0,
        type: 'multiGraph',
        options: {
          graph_data:
            componentData.graphData?.map(d => ({
              device_id: d.deviceId,
              sub_property: d.subProperty ?? undefined,
            })) ?? [],
          axis_mode: componentData.axisMode ?? 'merged',
          name: componentData.name,
        },
      } satisfies MultiGraphDashboardItem;
    } else if (componentData.type === 'alertList') {
      newItem = {
        cols: 3,
        rows: 3,
        y: 0,
        x: 0,
        type: 'alertList',
        options: {
          dataSourceId: componentData.dataSourceId!,
          name: componentData.name,
        },
      } satisfies AlertListDashboardItem;
    } else if (componentData.type === 'latestValue') {
      const data = componentData.graphData?.[0];
      if (!data) {
        return;
      }
      newItem = {
        cols: 2,
        rows: 2,
        y: 0,
        x: 0,
        type: 'latestValue',
        options: {
          deviceId: data.deviceId,
          subProperty: data.subProperty || undefined,
          name: componentData.name,
        },
      } satisfies LatestValueDashboardItem;
    } else if (componentData.type === 'compass') {
      const data = componentData.graphData?.[0];
      if (!data) {
        return;
      }
      newItem = {
        cols: 3,
        rows: 3,
        y: 0,
        x: 0,
        type: 'compass',
        options: {
          deviceId: data.deviceId,
          subProperty: data.subProperty || undefined,
          name: componentData.name,
        },
      } satisfies CompassDashboardItem;
    } else {
      throw new Error('Invalid component type');
    }

    // Simple positioning - add to the end
    this.dashboard.update(items => [...items, newItem]);
    this.saveDashboard(null);
  }

  removeItem($event: MouseEvent | TouchEvent, item: DashboardItem): void {
    $event.preventDefault();
    $event.stopPropagation();
    this.confirmationService.confirm({
      header: 'Remove Item',
      message: 'Are you sure you want to remove this item?',
      accept: () => {
        this.dashboard.update(items => items.filter(i => i !== item));
        this.saveDashboard(null);
      },
    });
  }
}
