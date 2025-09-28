import { Component, inject, signal } from '@angular/core';
import {
  GridsterConfig,
  GridsterItem,
  GridsterItemComponent,
  GridsterModule,
} from 'angular-gridster2';
import { ConfirmationService } from 'primeng/api';
import { ButtonModule } from 'primeng/button';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { AlertListComponent } from '../shared/alert/alert-list.component';
import { GraphComponent, GraphConfig } from '../shared/graph/graph.component';
import {
  AddDashboardComponentDialogComponent,
  DashboardComponentCreate,
} from './add-dashboard-component-dialog.component';

// Define discriminated union types for dashboard items
interface GraphDashboardItem extends GridsterItem {
  type: 'graph';
  options: GraphConfig & { name: string };
}

interface AlertListDashboardItem extends GridsterItem {
  type: 'alertList';
  options: {
    dataSourceId: string;
    name: string;
  };
}

// Union type for all dashboard items
type DashboardItem = GraphDashboardItem | AlertListDashboardItem;

@Component({
  selector: 'grn-dashboard',
  standalone: true,
  imports: [
    NavBarComponent,
    GridsterModule,
    GridsterItemComponent,
    GraphComponent,
    AlertListComponent,
    ButtonModule,
    AddDashboardComponentDialogComponent,
    ConfirmDialogModule,
  ],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
  providers: [ConfirmationService],
})
export class DashboardComponent {
  options: GridsterConfig;
  dashboard: DashboardItem[];
  dialogVisible = signal(false);
  editMode = signal(false);
  private confirmationService = inject(ConfirmationService);

  constructor() {
    this.options = {
      draggable: {
        enabled: true,
        stop: this.saveDashboard.bind(this), // called after drag
      },
      resizable: {
        enabled: true,
        stop: this.saveDashboard.bind(this), // called after resize
      },
      pushResizeItems: true,
      swap: true,
      swapWhileDragging: true,
      pushItems: true,
      minCols: 16,
      minRows: 9,
      maxCols: 16,
      maxRows: 9,
      disableScrollHorizontal: true,
      disableScrollVertical: true,
    };

    const savedLayout = localStorage.getItem('dashboard-layout');
    if (savedLayout) {
      console.log('loading dashboard');
      console.log(savedLayout);
      this.dashboard = JSON.parse(savedLayout);
    } else {
      console.log('no saved dashboard, using default');
      this.dashboard = [];
    }
  }

  saveDashboard(e: any) {
    setTimeout(() => {
      localStorage.setItem('dashboard-layout', JSON.stringify(this.dashboard));
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
          device_id: componentData.deviceId!,
          sub_property: componentData.subProperty!,
          name: componentData.name,
        },
      } as GraphDashboardItem;
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
