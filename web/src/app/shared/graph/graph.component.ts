import { CommonModule } from '@angular/common';
import { Component, computed, inject, input, signal } from '@angular/core';
import { GraphService } from './graph.service';
import { toSignal, toObservable } from '@angular/core/rxjs-interop';
import { catchError, map, switchMap } from 'rxjs/operators';
import { CardComponent } from '../../card/card.component';
import { ChartModule } from 'primeng/chart';
import { chartOptions } from './chart_option';
import { combineLatest, of } from 'rxjs';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { MessageModule } from 'primeng/message';
import { differenceInSeconds, subHours } from 'date-fns';
import { FormsModule } from '@angular/forms';
import { DatePickerModule } from 'primeng/datepicker';
export interface GraphConfig {
  device_id: string;
  sub_property?: string;
}

const MAX_DATA_POINTS = 100;

@Component({
  selector: 'grn-graph',
  standalone: true,
  imports: [
    CommonModule,
    CardComponent,
    ChartModule,
    ProgressSpinnerModule,
    MessageModule,
    FormsModule,
    DatePickerModule,
  ],
  templateUrl: './graph.component.html',
  styleUrl: './graph.component.scss',
})
export class GraphComponent {
  public config = input.required<GraphConfig>();
  private graphService = inject(GraphService);
  public chartOptions = chartOptions;

  public loading = signal(false);
  public error = signal<string | null>(null);

  public startDate = signal(subHours(new Date(), 1));
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

  // Convert the device signal to an observable, then switch to the timeseries observable
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
          return this.graphService
            .getTimeseries(deviceConfig.device_id, {
              start: startDate,
              end: endDate,
              step: steps,
            })
            .pipe(
              catchError(error => {
                this.loading.set(false);
                this.error.set(error.message);
                return of([]);
              })
            );
        })
      )
      .pipe(
        map(timeseries => {
          this.loading.set(false);
          return {
            labels: timeseries.map(t =>
              new Date(t.timestamp).toLocaleTimeString()
            ),
            datasets: [
              {
                data: timeseries.map(
                  t =>
                    (t.value as { Number: number }).Number + Math.random() * 100
                ),
              },
            ],
          };
        })
      )
  );
}
