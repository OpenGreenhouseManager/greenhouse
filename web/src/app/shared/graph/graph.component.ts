import { Component, computed, inject, input, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { NgxEchartsDirective } from 'ngx-echarts';
import { DatePickerModule } from 'primeng/datepicker';
import { DialogModule } from 'primeng/dialog';
import { MessageModule } from 'primeng/message';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { SelectButtonModule } from 'primeng/selectbutton';
import { CardComponent } from '../../card/card.component';
import {
  injectTimeseries,
  pickStep,
  stepMs,
  type TimeRange,
  TimeseriesStore,
} from '../timeseries/timeseries.store';
import { chartColors, type LineSeries, lineChartOptions } from './chart_option';

export interface GraphConfig {
  graph_data: GraphData[];
  axis_mode?: 'merged' | 'separate';
}

export interface GraphData {
  device_id: string;
  sub_property?: string | undefined;
}

const MAX_DATA_POINTS = 500;
const DEFAULT_WINDOW_MS = 24 * 60 * 60 * 1000;
// the date picker has minute precision, an end within this is treated as "now"
const LIVE_TOLERANCE_MS = 60_000;

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
  public dialogVisible = false;

  public now = inject(TimeseriesStore).now;

  // A live range slides with the store clock, a fixed range never refreshes
  public range = signal<TimeRange>({
    kind: 'live',
    windowMs: DEFAULT_WINDOW_MS,
  });

  public startDate = computed(() => {
    const range = this.range();
    return range.kind === 'live'
      ? new Date(this.now().getTime() - range.windowMs)
      : range.start;
  });

  public endDate = computed(() => {
    const range = this.range();
    return range.kind === 'live' ? this.now() : range.end;
  });

  public steps = computed(() => {
    const range = this.range();
    const duration =
      range.kind === 'live'
        ? range.windowMs
        : range.end.getTime() - range.start.getTime();
    return pickStep(duration, MAX_DATA_POINTS);
  });

  private series = injectTimeseries(() =>
    this.config().graph_data.map(s => ({
      device_id: s.device_id,
      sub_property: s.sub_property,
      step: this.steps(),
      range: this.range(),
    }))
  );

  public loading = computed(() => this.series().some(s => s.loading));

  public error = computed(() => {
    const errors = this.series()
      .filter(s => s.error)
      .map(s => `${seriesName(s.query)}: ${s.error}`);
    return errors.length ? errors.join('\n') : null;
  });

  public timeseries = computed(() =>
    this.series().map((s, idx): LineSeries => ({
      name: seriesName(s.query) || `Series ${idx + 1}`,
      unit: s.unit,
      color: palette[idx % palette.length] ?? chartColors.primary,
      data: s.points.map(([timestamp, value]) => [
        timestamp,
        typeof value === 'number' ? value : null,
      ]),
    }))
  );

  public options = computed(() =>
    lineChartOptions(this.timeseries(), this.config().axis_mode === 'separate')
  );

  public setStart(start: Date | null) {
    const end = this.endDate();
    if (!start || start >= end) return;
    this.setRange(start, end);
  }

  public setEnd(end: Date | null) {
    const start = this.startDate();
    if (!end || end <= start) return;
    this.setRange(start, end);
  }

  private setRange(start: Date, end: Date) {
    const windowMs = end.getTime() - start.getTime();
    const tolerance = Math.max(
      stepMs(pickStep(windowMs, MAX_DATA_POINTS)),
      LIVE_TOLERANCE_MS
    );
    this.range.set(
      Date.now() - end.getTime() <= tolerance
        ? { kind: 'live', windowMs }
        : { kind: 'fixed', start, end }
    );
  }
}

function seriesName(s: GraphData): string {
  return s.sub_property || s.device_id;
}
