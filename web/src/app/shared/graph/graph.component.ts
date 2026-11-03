import {
  Component,
  computed,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { NgxEchartsDirective } from 'ngx-echarts';
import { DatePickerModule } from 'primeng/datepicker';
import { DialogModule } from 'primeng/dialog';
import { MessageModule } from 'primeng/message';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
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
const HOUR_MS = 60 * 60 * 1000;
const DEFAULT_WINDOW_MS = 24 * HOUR_MS;
// the date picker has minute precision, an end within this is treated as "now"
const LIVE_TOLERANCE_MS = 60_000;

/** One-tap live ranges, the date pickers stay available for anything else. */
export const RANGE_PRESETS = [
  { label: '1h', windowMs: HOUR_MS },
  { label: '6h', windowMs: 6 * HOUR_MS },
  { label: '24h', windowMs: 24 * HOUR_MS },
  { label: '7d', windowMs: 7 * 24 * HOUR_MS },
  { label: '30d', windowMs: 30 * 24 * HOUR_MS },
] as const;

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
  ],
  templateUrl: './graph.component.html',
  styleUrl: './graph.component.scss',
})
export class GraphComponent {
  public config = input.required<GraphConfig>();
  public header = input.required<string>();
  /** Shows a maximize / restore button in the header. */
  public maximizable = input<boolean>(false);
  /** Whether the graph is currently shown maximized (flips the button). */
  public maximized = input<boolean>(false);
  public maximizeToggle = output();
  public dialogVisible = false;

  public now = inject(TimeseriesStore).now;

  public readonly presets = RANGE_PRESETS;

  // Touch devices get the date pickers as a modal with a read-only input:
  // a popup anchored to a tiny input does not fit on a phone, and focusing a
  // small input makes iOS zoom the page.
  public readonly touch = window.matchMedia('(pointer: coarse)').matches;

  private customRangeOpen = signal(false);

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

  /** The preset matching the current range, null for a custom range. */
  public activePreset = computed(() => {
    const range = this.range();
    if (range.kind !== 'live') return null;
    return this.presets.find(p => p.windowMs === range.windowMs) ?? null;
  });

  /** Custom ranges always show their dates, presets only on request. */
  public showCustomRange = computed(
    () => this.customRangeOpen() || this.activePreset() === null
  );

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

  public selectPreset(windowMs: number) {
    this.range.set({ kind: 'live', windowMs });
    this.customRangeOpen.set(false);
  }

  public toggleCustomRange() {
    this.customRangeOpen.update(open => !open);
  }

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
