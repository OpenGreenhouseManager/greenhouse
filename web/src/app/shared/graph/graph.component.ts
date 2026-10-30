import { Component, computed, inject, input, signal } from '@angular/core';
import { toObservable, toSignal } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { differenceInSeconds, subDays } from 'date-fns';
import { NgxEchartsDirective } from 'ngx-echarts';
import { DatePickerModule } from 'primeng/datepicker';
import { DialogModule } from 'primeng/dialog';
import { MessageModule } from 'primeng/message';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { SelectButtonModule } from 'primeng/selectbutton';
import { combineLatest, forkJoin, of } from 'rxjs';
import { catchError, map, switchMap } from 'rxjs/operators';
import { CardComponent } from '../../card/card.component';
import { chartColors, type LineSeries, lineChartOptions } from './chart_option';
import { GraphService } from './graph.service';

export interface GraphConfig {
  graph_data: GraphData[];
  axis_mode?: 'merged' | 'separate';
}

export interface GraphData {
  device_id: string;
  sub_property?: string | undefined;
}

const MAX_DATA_POINTS = 100;

const palette = [
  chartColors.primary,
  chartColors.secondary,
  chartColors.info,
  chartColors.warning,
  chartColors.success,
  chartColors.error,
  chartColors.fatal,
];

@Component({
  selector: 'grn-graph',
  imports: [
    CardComponent,
    NgxEchartsDirective,
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
      return `${Math.floor(step_count)}s`;
    } else if (step_count < 3600) {
      return `${Math.floor(step_count / 60)}m`;
    } else if (step_count < 86400) {
      return `${Math.floor(step_count / 3600)}h`;
    }
    return '1d';
  });

  public now = signal(new Date());

  // Convert inputs to an observable, then fetch one or more timeseries and map them to chart series
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
          if (!responses) {
            return [];
          }

          return responses.map((r, idx): LineSeries => {
            // Determine unit (if Measurement) for this series
            const firstMeasurement = r.timeseries.find(
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              t => (t.value as any)?.Measurement
            );
            const unit: string =
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              (firstMeasurement?.value as any)?.Measurement?.unit ?? '';

            // Map value, supporting Number and Measurement; null for others
            const data = r.timeseries.map((t): [number, number | null] => {
              const v = t.value;
              let value: number | null = null;
              if (v && typeof v === 'object') {
                if ('Measurement' in v && v.Measurement) {
                  value = v.Measurement.value as number;
                } else if ('Number' in v) {
                  value = v.Number as number;
                }
              }
              return [t.timestamp * 1000, value];
            });

            const graphData = this.config().graph_data[idx];
            return {
              name:
                graphData?.sub_property ||
                graphData?.device_id ||
                `Series ${idx + 1}`,
              unit,
              color: palette[idx % palette.length] ?? chartColors.primary,
              data,
            };
          });
        })
      ),
    { initialValue: [] as LineSeries[] }
  );

  public options = computed(() =>
    lineChartOptions(this.timeseries(), this.config().axis_mode === 'separate')
  );
}
