import { Component, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed, toObservable } from '@angular/core/rxjs-interop';
import { ConfirmationService } from 'primeng/api';
import { catchError, debounceTime, EMPTY, filter, skip, switchMap } from 'rxjs';
import { ButtonDirective } from 'primeng/button';
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
  imports: [
    NavBarComponent,
    ButtonDirective,
    AddDashboardComponentDialogComponent,
    ConfirmDialogModule,
    TabsModule,
    DashboardTabComponent,
  ],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss',
  providers: [ConfirmationService],
})
export class DashboardComponent {
  private userPreferencesService = inject(UserPreferencesService);
  private confirmationService = inject(ConfirmationService);

  dashboard = signal<DashboardConfig | undefined>(undefined);
  dialogVisible = signal(false);
  editMode = signal(false);
  activeTab = signal(0);

  /** The widget open in the edit dialog, null while adding a new one. */
  editing = signal<{ tabIndex: number; item: DashboardItem } | null>(null);
  editingData = computed(() => {
    const editing = this.editing();
    return editing ? toComponentData(editing.item) : null;
  });

  constructor() {
    this.loadDashboard();

    // Persist changes to the backend, debounced. skip(1) drops the value that
    // was just loaded, so opening the dashboard does not write it back.
    toObservable(this.dashboard)
      .pipe(
        filter(dashboard => dashboard !== undefined),
        skip(1),
        debounceTime(300),
        switchMap(dashboard =>
          this.userPreferencesService.getUserPreferences().pipe(
            switchMap(currentPreferences =>
              this.userPreferencesService.updateUserPreferences({
                ...currentPreferences,
                dashboard_preferences: JSON.stringify(dashboard),
              })
            ),
            // keep the stream alive so later changes are still saved
            catchError(() => EMPTY)
          )
        ),
        takeUntilDestroyed()
      )
      .subscribe();
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
    this.activeTab.set(dashboard.length);
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
        if (this.activeTab() >= dashboard.length - 1) {
          this.activeTab.set(Math.max(0, dashboard.length - 2));
        }
      },
    });
  }

  onDialogClose() {
    this.dialogVisible.set(false);
    this.editing.set(null);
  }

  onComponentAdded(componentData: DashboardComponentCreate) {
    const newItem = createItem(componentData);
    if (!newItem) {
      return;
    }

    // add to the end of the active tab. Build new tab and items references:
    // grn-dashboard-tab is OnPush and only re-renders on a new input value.
    this.dashboard.update(tabs => {
      if (!tabs) {
        return undefined;
      }
      const activeIndex = this.activeTab();
      const activeTab = tabs[activeIndex] ?? {
        name: componentData.name,
        items: [],
      };
      const newTabs = [...tabs];
      newTabs[activeIndex] = {
        ...activeTab,
        items: [...activeTab.items, newItem],
      };
      return newTabs;
    });
  }

  onEditItem(tabIndex: number, item: DashboardItem) {
    this.editing.set({ tabIndex, item });
    this.dialogVisible.set(true);
  }

  /** Replaces the edited widget's settings, keeping its place on the grid. */
  onComponentUpdated(componentData: DashboardComponentCreate) {
    const editing = this.editing();
    const updated = createItem(componentData);
    if (!editing || !updated || updated.type !== editing.item.type) {
      return;
    }
    const { x, y, cols, rows } = editing.item;
    const newItem = { ...editing.item, ...updated, x, y, cols, rows };
    this.dashboard.update(tabs => {
      const tab = tabs?.[editing.tabIndex];
      if (!tabs || !tab) {
        return tabs;
      }
      const newTabs = [...tabs];
      newTabs[editing.tabIndex] = {
        ...tab,
        items: tab.items.map(i => (i === editing.item ? newItem : i)),
      };
      return newTabs;
    });
    this.editing.set(null);
  }

  onDashboardChange(index: number, dashboard: DashboardItem[]) {
    this.dashboard.update(tabs => {
      const currentTab = tabs?.[index];
      if (!tabs || !currentTab) {
        return tabs;
      }
      const updatedTabs = [...tabs];
      updatedTabs[index] = { ...currentTab, items: dashboard };
      return updatedTabs;
    });
  }
}

/** Builds a new widget, placed at the top left, from the dialog's data. */
function createItem(
  componentData: DashboardComponentCreate
): DashboardItem | undefined {
  switch (componentData.type) {
    case 'graph': {
      const data = componentData.graphData?.[0];
      if (!data) {
        return undefined;
      }
      return {
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
    }
    case 'multiGraph':
      return {
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
    case 'alertList':
      if (!componentData.dataSourceId) {
        return undefined;
      }
      return {
        cols: 3,
        rows: 3,
        y: 0,
        x: 0,
        type: 'alertList',
        options: {
          dataSourceId: componentData.dataSourceId,
          name: componentData.name,
        },
      } satisfies AlertListDashboardItem;
    case 'latestValue': {
      const data = componentData.graphData?.[0];
      if (!data) {
        return undefined;
      }
      return {
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
    }
    case 'compass': {
      const data = componentData.graphData?.[0];
      if (!data) {
        return undefined;
      }
      return {
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
    }
  }
}

/** The dialog's view of an existing widget, to prefill the edit form. */
export function toComponentData(item: DashboardItem): DashboardComponentCreate {
  switch (item.type) {
    case 'graph':
    case 'multiGraph':
      return {
        type: item.type,
        name: item.options.name,
        axisMode: item.options.axis_mode ?? 'merged',
        graphData: item.options.graph_data.map(d => ({
          deviceId: d.device_id,
          subProperty: d.sub_property ?? '',
        })),
      };
    case 'alertList':
      return {
        type: item.type,
        name: item.options.name,
        dataSourceId: item.options.dataSourceId,
      };
    case 'latestValue':
    case 'compass':
      return {
        type: item.type,
        name: item.options.name,
        graphData: [
          {
            deviceId: item.options.deviceId,
            subProperty: item.options.subProperty ?? '',
          },
        ],
      };
  }
}
