import { Component, computed, inject, input, signal } from '@angular/core';
import { toObservable, toSignal } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import {
  differenceInCalendarDays,
  differenceInSeconds,
  fromUnixTime,
  subDays,
} from 'date-fns';
import { ChartModule } from 'primeng/chart';
import { DatePickerModule } from 'primeng/datepicker';
import { DialogModule } from 'primeng/dialog';
import { MessageModule } from 'primeng/message';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { SelectButtonModule } from 'primeng/selectbutton';
import { combineLatest, forkJoin, of } from 'rxjs';
import { catchError, map, switchMap } from 'rxjs/operators';
import { CardComponent } from '../../card/card.component';
import { chartColors, chartOptions } from './chart_option';
import { GraphService } from './graph.service';

export interface GraphConfig {
  graph_data: GraphData[];
  axis_mode?: 'merged' | 'separate';
}

export interface GraphData {
  device_id: string;
  sub_property?: string;
}

const MAX_DATA_POINTS = 100;

@Component({
  selector: 'grn-graph',
  standalone: true,
  imports: [
    CardComponent,
    ChartModule,
    ProgressSpinnerModule,
    MessageModule,
    FormsModule,
    DatePickerModule,
    DialogModule,
    SelectButtonModule,
  ],
  templateUrl: './graph.component.html',
  styleUrl: './graph.component.scss',
})
export class GraphComponent {
  public config = input.required<GraphConfig>();
  public header = input.required<string>();
  private graphService = inject(GraphService);
  public loading = signal(false);
  public error = signal<string | null>(null);
  public dialogVisible = false;
  public startDate = signal(subDays(new Date(), 1));
  public endDate = signal(new Date());

  // calculate the step size based on the duration and the max data points
  // output should be prometeus steps example 1s, 5s, 10s, 15s, 30s, 1m, 2m, 5m, 10m, 1h, 1d
  public steps = computed(() => {
    const raw_pint_count =
      differenceInSeconds(this.endDate(), this.startDate()) / 5;
    const step_count = raw_pint_count / MAX_DATA_POINTS;

    if (step_count < 1) {
      return '1s';
    } else if (step_count < 60) {
      return Math.floor(step_count) + 's';
    } else if (step_count < 3600) {
      return Math.floor(step_count / 60) + 'm';
    } else if (step_count < 86400) {
      return Math.floor(step_count / 3600) + 'h';
    } else {
      return '1d';
    }
  });

  public now = signal(new Date());

  // Chart.js options computed from current state
  public options = computed(() => {
    const base = structuredClone(chartOptions) as any;
    // Enable legend if more than one dataset will be shown
    base.plugins = base.plugins || {};
    base.plugins.legend = base.plugins.legend || {};

    // Configure secondary Y axis if requested
    base.scales = base.scales || {};
    base.scales.x = base.scales.x || {};
    base.scales.y = {
      ...(base.scales.y || {}),
      type: 'linear',
      position: 'left',
      title: {
        ...(base.scales.y?.title || {}),
        display: false,
        text: '',
      },
    };
    // Formatter to round numbers to at most two decimal places
    const formatNumber = (val: unknown): string => {
      if (val === null || val === undefined) return '';
      const num =
        typeof val === 'number'
          ? val
          : typeof val === 'string'
            ? Number(val)
            : NaN;
      if (!Number.isFinite(num)) return String(val as any);
      return new Intl.NumberFormat(undefined, {
        maximumFractionDigits: 2,
      }).format(num);
    };

    // Apply tick formatter on primary Y axis
    base.scales.y.ticks = {
      ...(base.scales.y?.ticks || {}),
      callback: (value: unknown) => formatNumber(value),
    };
    if (this.config().axis_mode === 'separate') {
      base.scales.y1 = {
        type: 'linear',
        position: 'right',
        grid: {
          drawOnChartArea: false,
        },
        ticks: {
          ...(base.scales.y?.ticks || {}),
          callback: (value: unknown) => formatNumber(value),
        },
        title: {
          display: false,
          text: '',
        },
      };
    } else {
      // Ensure single axis mode
      if (base.scales.y1) {
        delete base.scales.y1;
      }
    }

    // Derive units from current datasets and set axis titles accordingly
    const series = (this.timeseries?.() as any) || { datasets: [] };
    const datasets = Array.isArray(series.datasets) ? series.datasets : [];
    const unitsSet = new Set(
      datasets
        .map((d: any) => d?.unit)
        .filter((u: string | undefined) => !!u && typeof u === 'string')
    ) as Set<string>;

    // Show legend only if we actually have more than one dataset
    base.plugins.legend.display = datasets.length > 1;

    // Tooltip callback to append units to values
    base.plugins.tooltip = base.plugins.tooltip || {};
    base.plugins.tooltip.callbacks = base.plugins.tooltip.callbacks || {};
    base.plugins.tooltip.callbacks.label = (context: any) => {
      const ds = context.dataset || {};
      const unit = ds.unit ? ` ${ds.unit}` : '';
      const value = context.parsed?.y ?? context.raw;
      const label = ds.label ? `${ds.label}: ` : '';
      return `${label}${formatNumber(value)}${unit}`;
    };

    if (this.config().axis_mode === 'separate') {
      const left = datasets.find((d: any) => d?.yAxisID === 'y');
      const right = datasets.find((d: any) => d?.yAxisID === 'y1');
      if (left?.unit) {
        base.scales.y.title.display = true;
        base.scales.y.title.text = left.unit;
      } else {
        base.scales.y.title.display = false;
        base.scales.y.title.text = '';
      }
      if (base.scales.y1) {
        if (right?.unit) {
          base.scales.y1.title.display = true;
          base.scales.y1.title.text = right.unit;
        } else {
          base.scales.y1.title.display = false;
          base.scales.y1.title.text = '';
        }
      }
    } else {
      // merged axis: only show unit if all defined units are identical
      if (unitsSet.size === 1) {
        const [unit] = Array.from(unitsSet);
        base.scales.y.title.display = true;
        base.scales.y.title.text = unit;
      } else {
        base.scales.y.title.display = false;
        base.scales.y.title.text = '';
      }
    }
    return base;
  });

  // Convert inputs to an observable, then fetch one or more timeseries and map to Chart.js dataset structure
  public timeseries = toSignal(
    combineLatest([
      toObservable(this.config),
      toObservable(this.startDate),
      toObservable(this.endDate),
      toObservable(this.steps),
    ])
      .pipe(
        switchMap(([deviceConfig, startDate, endDate, steps]) => {
          this.loading.set(true);
          this.error.set(null);

          return forkJoin(
            deviceConfig.graph_data.map(s =>
              this.graphService.getTimeseries(s.device_id, {
                start: startDate,
                end: endDate,
                step: steps,
                sub_property: s.sub_property,
              })
            )
          ).pipe(
            map(responses => ({ responses })),
            catchError(error => {
              this.loading.set(false);
              this.error.set(error.message);
              return of({ responses: [] });
            })
          );
        })
      )
      .pipe(
        map(({ responses }) => {
          this.loading.set(false);
          // If no data, return empty chart
          if (
            !responses ||
            (Array.isArray(responses) && responses.length === 0)
          ) {
            return { labels: [], datasets: [] };
          }

          const isMultiDayRange =
            differenceInCalendarDays(this.endDate(), this.startDate()) >= 1;
          const labels =
            responses[0].timeseries.map(t => {
              const d = fromUnixTime(t.timestamp);
              return isMultiDayRange
                ? d.toLocaleString(undefined, {
                    month: 'short',
                    day: '2-digit',
                    hour: '2-digit',
                    minute: '2-digit',
                    hour12: false,
                  })
                : d.toLocaleTimeString(undefined, {
                    hour: '2-digit',
                    minute: '2-digit',
                    hour12: false,
                  });
            }) ?? [];

          const palette = [
            chartColors.primary,
            chartColors.secondary,
            chartColors.info,
            chartColors.warning,
            chartColors.success,
            chartColors.error,
            chartColors.fatal,
          ];

          const useSeparateAxes = this.config().axis_mode === 'separate';

          const datasets = responses.map((r, idx) => {
            // Determine unit (if Measurement) for this dataset
            const firstMeasurement = r.timeseries.find(
              t => (t.value as any)?.Measurement
            ) as any;
            const unit: string =
              firstMeasurement?.value?.Measurement?.unit ??
              firstMeasurement?.Measurement?.unit ??
              '';

            // Map value, supporting Number and Measurement; null for others
            const data = r.timeseries.map(t => {
              const v = t.value as any;
              if (v && typeof v === 'object') {
                if ('Measurement' in v && v.Measurement) {
                  return v.Measurement.value as number;
                }
                if ('Number' in v) {
                  return v.Number as number;
                }
              }
              return null;
            });
            const color = palette[idx % palette.length];
            let label =
              (this.config().graph_data[idx].sub_property
                ? `${this.config().graph_data[idx].sub_property}`
                : this.config().graph_data[idx].device_id) ||
              `Series ${idx + 1}`;
            if (unit) {
              label = `${label} (${unit})`;
            }
            return {
              label: label || `Series ${idx + 1}`,
              data,
              borderColor: color,
              backgroundColor: color + '33',
              yAxisID: useSeparateAxes ? (idx === 0 ? 'y' : 'y1') : 'y',
              unit,
            };
          });

          return { labels, datasets };
        })
      )
  );
}
