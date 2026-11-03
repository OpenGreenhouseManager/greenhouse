import {
  Component,
  computed,
  DestroyRef,
  inject,
  input,
  model,
  output,
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

/** Widgets that can be shown maximized over the dashboard. */
export type GraphLikeDashboardItem =
  GraphDashboardItem | MultiGraphDashboardItem;

export type DashboardItem =
  | GraphDashboardItem
  | MultiGraphDashboardItem
  | AlertListDashboardItem
  | LatestValueDashboardItem
  | CompassDashboardItem;

/**
 * Below this viewport width the free 16-column layout gets too narrow to be
 * readable, so gridster stacks the widgets and CSS reflows them into as many
 * columns as fit (see dashboard-tab.component.scss).
 */
export const COMPACT_BREAKPOINT_PX = 1024;

/** Height of one grid row in the compact layout, widgets keep their `rows`. */
const COMPACT_ROW_HEIGHT_PX = 100;

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
    '(document:keydown.escape)': 'restore()',
  },
})
export class DashboardTabComponent {
  confirmationService = inject(ConfirmationService);

  dashboard = model.required<DashboardItem[]>();
  editMode = input.required<boolean>();

  /** Asks the parent to open the edit dialog for this widget. */
  editItem = output<DashboardItem>();

  private maximized = signal<GraphLikeDashboardItem | null>(null);

  /**
   * The graph shown over the whole dashboard. Purely view state: it is not
   * saved, and it closes in edit mode or when its widget goes away.
   */
  maximizedItem = computed(() => {
    const item = this.maximized();
    if (!item || this.editMode()) {
      return null;
    }
    // close when the widget was removed or replaced
    return this.dashboard().includes(item) ? item : null;
  });

  /** True while gridster shows the stacked layout instead of the grid. */
  compact = viewportBelow(COMPACT_BREAKPOINT_PX);

  /**
   * The stacked layout follows DOM order, so render the widgets in reading
   * order of the grid layout (top to bottom, left to right).
   */
  items = computed(() => {
    const items = this.dashboard();
    if (!this.compact()) {
      return items;
    }
    return [...items].sort((a, b) => a.y - b.y || a.x - b.x);
  });

  // Gridster's `options` is a signal input, so a change must hand it a new
  // object; mutating the old one is not picked up.
  options = computed<GridsterConfig>(() => ({
    gridType: GridType.ScrollVertical,
    // Only drag and resize in edit mode: otherwise touching a widget starts a
    // drag instead of scrolling the page or interacting with the chart.
    draggable: {
      enabled: this.editMode(),
      stop: () => {
        //trigger change detection
        this.dashboard.update(items => [...items]);
      },
    },
    resizable: {
      enabled: this.editMode(),
      stop: () => {
        //trigger change detection
        this.dashboard.update(items => [...items]);
      },
    },
    mobileBreakpoint: COMPACT_BREAKPOINT_PX,
    useBodyForBreakpoint: true,
    keepFixedHeightInMobile: true,
    fixedRowHeight: COMPACT_ROW_HEIGHT_PX,
    pushResizeItems: true,
    swap: true,
    swapWhileDragging: true,
    minCols: 16,
    minRows: 9,
    maxCols: 16,
    disableScrollHorizontal: true,
    disableScrollVertical: false,
  }));

  maximize(item: GraphLikeDashboardItem): void {
    this.maximized.set(item);
  }

  restore(): void {
    this.maximized.set(null);
  }

  onEditItem($event: Event, item: DashboardItem): void {
    $event.preventDefault();
    $event.stopPropagation();
    this.editItem.emit(item);
  }

  removeItem($event: Event, item: DashboardItem): void {
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

/** Signal that tracks whether the viewport is narrower than `px`. */
function viewportBelow(px: number) {
  const query = window.matchMedia(`(max-width: ${px - 0.02}px)`);
  const matches = signal(query.matches);
  const onChange = (event: MediaQueryListEvent) => matches.set(event.matches);
  query.addEventListener('change', onChange);
  inject(DestroyRef).onDestroy(() =>
    query.removeEventListener('change', onChange)
  );
  return matches.asReadonly();
}
