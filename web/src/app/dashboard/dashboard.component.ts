import { Component, effect, inject, OnInit, signal } from '@angular/core';
import { ConfirmationService } from 'primeng/api';
import { ButtonModule } from 'primeng/button';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { TabsModule } from 'primeng/tabs';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { UserPreferencesService } from '../services/user-preferences.service';
import type { DashboardComponentCreate } from './add-dashboard-component-dialog.component';
import { AddDashboardComponentDialogComponent } from './add-dashboard-component-dialog.component';
import {
  AlertListDashboardItem,
  CompassDashboardItem,
  DashboardItem,
  DashboardTabComponent,
  GraphDashboardItem,
  LatestValueDashboardItem,
  MultiGraphDashboardItem,
} from './dashboard-tab.component';

// Union type for all dashboard items

type DashboardConfig = {
  name: string;
  items: DashboardItem[];
}[];

@Component({
  selector: 'grn-dashboard',
  standalone: true,
  imports: [
    NavBarComponent,
    ButtonModule,
    AddDashboardComponentDialogComponent,
    ConfirmDialogModule,
    TabsModule,
    DashboardTabComponent,
  ],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
  providers: [ConfirmationService],
})
export class DashboardComponent implements OnInit {
  private userPreferencesService = inject(UserPreferencesService);
  private confirmationService = inject(ConfirmationService);

  dashboard = signal<DashboardConfig | undefined>(undefined);
  dialogVisible = signal(false);
  editMode = signal(false);
  activeTab = signal(0);

  ngOnInit(): void {
    this.loadDashboard();
  }

  constructor() {
    effect(() => {
      if (!this.dashboard()) {
        return;
      }
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
    });
  }

  private loadDashboard() {
    this.userPreferencesService.getUserPreferences().subscribe({
      next: preferences => {
        if (preferences.dashboard_preferences) {
          this.dashboard.set(JSON.parse(preferences.dashboard_preferences));
        } else {
          this.dashboard.set([{ name: 'Dashboard', items: [] }]);
        }
      },
      error: () => {
        //this.dashboard.set({ tabs: [{ name: 'Dashboard', items: [] }] });
      },
    });
  }

  addEntry() {
    this.dialogVisible.set(true);
  }

  addNewTab() {
    const dashboard = this.dashboard();
    if (!dashboard) {
      return;
    }
    const newTabNumber = dashboard.length + 1;
    const newTab = {
      name: `Tab ${newTabNumber}`,
      items: [],
    };
    this.dashboard.update(tabs => [...(tabs ?? []), newTab]);
    this.activeTab.set(dashboard.length - 1);
  }

  updateTabName(index: number, event: Event) {
    const input = event.target as HTMLInputElement;
    const newName = input.value;
    // Update the input size to match the content
    input.size = Math.max(1, newName.length);
    this.dashboard.update(tabs => {
      const updatedTabs = [...(tabs ?? [])];
      const currentTab = updatedTabs[index];
      if (currentTab) {
        updatedTabs[index] = {
          name: newName,
          items: currentTab.items,
        };
      }
      return updatedTabs;
    });
  }

  deleteTab(index: number, event: Event) {
    event.stopPropagation();
    const dashboard = this.dashboard();
    if (!dashboard) {
      return;
    }

    // Prevent deleting the last tab
    if (dashboard.length <= 1) {
      return;
    }

    this.confirmationService.confirm({
      message: 'Are you sure you want to delete this tab?',
      header: 'Delete Confirmation',
      icon: 'pi pi-exclamation-triangle',
      accept: () => {
        this.dashboard.update(tabs => {
          const updatedTabs = tabs?.filter((_, i) => i !== index) ?? [];
          return updatedTabs;
        });

        // Adjust active tab if necessary
        if (this.activeTab() >= dashboard.length) {
          this.activeTab.set(Math.max(0, dashboard.length - 1));
        }
      },
    });
  }

  onDialogClose() {
    this.dialogVisible.set(false);
  }

  onComponentAdded(componentData: DashboardComponentCreate) {
    const dashboard = this.dashboard();
    if (!dashboard) {
      return;
    }
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

    // add to the end of the active tab
    this.dashboard.update(tabs => {
      if (!tabs) {
        return undefined;
      }
      const newTabs = [...tabs];
      let activeTabs = newTabs[this.activeTab()];
      if (!activeTabs) {
        activeTabs = { name: componentData.name, items: [] };
      }
      activeTabs.items.push(newItem);
      tabs[this.activeTab()] = activeTabs;
      return newTabs;
    });
  }

  onDashboardChange(index: number, dashboard: DashboardItem[]) {
    this.dashboard.update(tabs => {
      if (!tabs) {
        return undefined;
      }
      const currentTab = tabs[index];
      if (!currentTab) {
        return undefined;
      }
      currentTab.items = dashboard;

      const updatedTabs = [...tabs];
      updatedTabs[index] = currentTab;

      return updatedTabs;
    });
  }
}
