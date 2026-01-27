import { Component, inject, OnDestroy, signal } from '@angular/core';
import {
  Gridster,
  GridsterConfig,
  GridsterItem,
  GridsterItemConfig,
  GridType,
} from 'angular-gridster2';
import { ConfirmationService } from 'primeng/api';
import { ButtonModule } from 'primeng/button';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { UserPreferencesService } from '../services/user-preferences.service';
import { AlertListComponent } from '../shared/alert/alert-list.component';
import { CompassCardComponent } from '../shared/compass/compass-card.component';
import { GraphComponent, GraphConfig } from '../shared/graph/graph.component';
import { LatestValueCardComponent } from '../shared/value/latest-value-card.component';
import {
  AddDashboardComponentDialogComponent,
  DashboardComponentCreate,
} from './add-dashboard-component-dialog.component';

// Define discriminated union types for dashboard items
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
    subProperty?: string;
    name: string;
  };
}

interface CompassDashboardItem extends GridsterItemConfig {
  type: 'compass';
  options: {
    deviceId: string;
    subProperty?: string;
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
export class DashboardComponent implements OnDestroy {
  options: GridsterConfig;
  dashboard: DashboardItem[] = [];
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

    window.addEventListener('resize', this.onWindowResize);

    // Load dashboard from backend preferences instead of localStorage
    this.loadDashboard();
  }

  private loadDashboard() {
    this.userPreferencesService.getUserPreferences().subscribe({
      next: preferences => {
        if (preferences.dashboard_preferences) {
          console.log('loading dashboard from backend');
          console.log(preferences.dashboard_preferences);
          this.dashboard = JSON.parse(preferences.dashboard_preferences);
          console.log('dashboard loaded from backend', this.dashboard);
        } else {
          console.log('no saved dashboard, using default');
          this.dashboard = [];
        }
      },
      error: error => {
        console.error('Error loading dashboard preferences:', error);
        this.dashboard = [];
      },
    });
  }

  ngOnDestroy() {
    window.removeEventListener('resize', this.onWindowResize);
  }

  saveDashboard(e: any) {
    setTimeout(() => {
      // Save to backend instead of localStorage
      this.userPreferencesService.getUserPreferences().subscribe({
        next: currentPreferences => {
          const updatedPreferences = {
            ...currentPreferences,
            dashboard_preferences: JSON.stringify(this.dashboard),
          };

          this.userPreferencesService
            .updateUserPreferences(updatedPreferences)
            .subscribe({
              next: () => {
                console.log('Dashboard preferences saved successfully');
              },
              error: error => {
                console.error('Error saving dashboard preferences:', error);
              },
            });
        },
        error: error => {
          console.error('Error getting current preferences:', error);
        },
      });
    }, 300);
  }

  addEntry() {
    this.dialogVisible.set(true);
    console.log('dialogVisible', this.dialogVisible());
  }

  onDialogClose() {
    this.dialogVisible.set(false);
  }

  onComponentAdded(componentData: DashboardComponentCreate) {
    // Create the appropriate dashboard item based on type
    let newItem: DashboardItem;

    if (componentData.type === 'graph') {
      newItem = {
        cols: 3,
        rows: 3,
        y: 0,
        x: 0,
        type: 'graph',
        options: {
          graph_data: [
            {
              device_id: componentData.deviceId!,
              sub_property: componentData.subProperty!,
            },
          ],
          name: componentData.name,
        },
      } as GraphDashboardItem;
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
              sub_property: d.subProperty,
            })) ?? [],
          axis_mode: componentData.axisMode ?? 'merged',
          name: componentData.name,
        },
      } as MultiGraphDashboardItem;
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
      } as AlertListDashboardItem;
    } else if (componentData.type === 'latestValue') {
      newItem = {
        cols: 2,
        rows: 2,
        y: 0,
        x: 0,
        type: 'latestValue',
        options: {
          deviceId: componentData.deviceId!,
          subProperty: componentData.subProperty || undefined,
          name: componentData.name,
        },
      } as LatestValueDashboardItem;
    } else if (componentData.type === 'compass') {
      newItem = {
        cols: 3,
        rows: 3,
        y: 0,
        x: 0,
        type: 'compass',
        options: {
          deviceId: componentData.deviceId!,
          subProperty: componentData.subProperty || undefined,
          name: componentData.name,
        },
      } as CompassDashboardItem;
    } else {
      throw new Error('Invalid component type');
    }

    // Simple positioning - add to the end
    this.dashboard.push(newItem);
    this.saveDashboard(null);
  }

  removeItem($event: MouseEvent | TouchEvent, item: DashboardItem): void {
    $event.preventDefault();
    $event.stopPropagation();
    this.confirmationService.confirm({
      header: 'Remove Item',
      message: 'Are you sure you want to remove this item?',
      accept: () => {
        this.dashboard.splice(this.dashboard.indexOf(item), 1);
        this.saveDashboard(null);
      },
    });
  }
}
