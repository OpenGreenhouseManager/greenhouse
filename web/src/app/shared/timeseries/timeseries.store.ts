import type { Signal, WritableSignal } from '@angular/core';
import {
  computed,
  DestroyRef,
  effect,
  inject,
  Service,
  signal,
  untracked,
} from '@angular/core';
import type {
  BatchTimeseriesResultDto,
  DeviceTimeseriesQueryDto,
  TimeseriesDto,
  Type,
} from '../../dtos/timeseries';
import { GraphService } from '../graph/graph.service';

// Prometheus step sizes the store queries with. Only fixed rungs are used so
// that widgets with similar ranges end up with the same query key.
const STEPS = [
  ['5s', 5],
  ['15s', 15],
  ['30s', 30],
  ['1m', 60],
  ['2m', 120],
  ['5m', 300],
  ['10m', 600],
  ['15m', 900],
  ['30m', 1800],
  ['1h', 3600],
  ['3h', 10800],
  ['6h', 21600],
  ['12h', 43200],
  ['1d', 86400],
] as const;

export type Step = (typeof STEPS)[number][0];

/** How often live queries are checked for a new step bucket. */
export const TICK_MS = 30_000;
/** Delay to collect registrations into a single batch request. */
export const FLUSH_DELAY_MS = 50;
/** Upper bound of queries per batch request (backend MAX_BATCH_QUERIES). */
export const MAX_BATCH_QUERIES = 50;

export type TimeRange =
  | { kind: 'live'; windowMs: number }
  | { kind: 'fixed'; start: Date; end: Date };

export interface SeriesQuery {
  device_id: string;
  sub_property?: string | undefined;
  step: Step;
  range: TimeRange;
}

export type SeriesValue = number | boolean | null;

export interface SeriesState {
  /** True until the first response for this query arrived. */
  loading: boolean;
  error: string | null;
  unit: string;
  // [unix ms, value]
  points: [number, SeriesValue][];
}

export type SeriesResult = SeriesState & { query: SeriesQuery };

export interface SeriesHandle {
  state: Signal<SeriesState>;
  release: () => void;
}

interface Entry {
  query: SeriesQuery;
  refs: number;
  state: WritableSignal<SeriesState>;
  // step bucket of the last request, live queries refresh when it changes
  bucket: number | null;
  // guards against an older response overwriting a newer one
  seq: number;
}

const INITIAL_STATE: SeriesState = {
  loading: true,
  error: null,
  unit: '',
  points: [],
};

export function stepMs(step: Step): number {
  return STEPS.find(([s]) => s === step)![1] * 1000;
}

/** Smallest step that keeps the range at or below `maxPoints` samples. */
export function pickStep(durationMs: number, maxPoints: number): Step {
  const raw = durationMs / 1000 / maxPoints;
  return (STEPS.find(([, seconds]) => seconds >= raw) ?? STEPS.at(-1)!)[0];
}

export function toValue(value: Type): SeriesValue {
  if ('Measurement' in value) return value.Measurement.value;
  if ('Number' in value) return value.Number;
  if ('Boolean' in value) return value.Boolean;
  return null;
}

function unitOf(timeseries: TimeseriesDto[]): string {
  for (const t of timeseries) {
    if ('Measurement' in t.value) return t.value.Measurement.unit;
  }
  return '';
}

function alignDown(ms: number, step: number): number {
  return Math.floor(ms / step) * step;
}

/** Concrete start/end for a query, aligned to the step. */
export function resolveRange(
  query: SeriesQuery,
  now: number
): { start: Date; end: Date } {
  const step = stepMs(query.step);
  const { range } = query;
  if (range.kind === 'live') {
    return {
      start: new Date(alignDown(now - range.windowMs, step)),
      end: new Date(now),
    };
  }
  return {
    start: new Date(alignDown(range.start.getTime(), step)),
    end: new Date(alignDown(range.end.getTime(), step)),
  };
}

export function queryKey(query: SeriesQuery): string {
  const { range } = query;
  const base = `${query.device_id}|${query.sub_property ?? ''}|${query.step}`;
  if (range.kind === 'live') {
    return `${base}|live:${range.windowMs}`;
  }
  const { start, end } = resolveRange(query, 0);
  return `${base}|${start.getTime()}|${end.getTime()}`;
}

/**
 * Shared cache for device timeseries. Identical queries are deduplicated,
 * pending queries are sent as one batch request and live queries are
 * refreshed on a shared clock.
 */
@Service()
export class TimeseriesStore {
  private graphService = inject(GraphService);

  private entries = new Map<string, Entry>();
  private dirty = new Set<string>();
  private flushTimer: ReturnType<typeof setTimeout> | null = null;
  private clock: ReturnType<typeof setInterval> | null = null;

  private readonly _now = signal(new Date());
  public readonly now = this._now.asReadonly();

  register(query: SeriesQuery): SeriesHandle {
    const key = queryKey(query);
    let entry = this.entries.get(key);
    if (!entry) {
      entry = {
        query,
        refs: 0,
        state: signal(INITIAL_STATE),
        bucket: null,
        seq: 0,
      };
      this.entries.set(key, entry);
      this.markDirty(key);
    }
    entry.refs++;
    this.startClock();

    let released = false;
    return {
      state: entry.state.asReadonly(),
      release: () => {
        if (released) return;
        released = true;
        this.release(key);
      },
    };
  }

  private release(key: string) {
    const entry = this.entries.get(key);
    if (!entry || --entry.refs > 0) return;
    this.entries.delete(key);
    this.dirty.delete(key);
    if (this.entries.size === 0) this.stopClock();
  }

  private startClock() {
    if (this.clock !== null) return;
    this._now.set(new Date());
    this.clock = setInterval(() => this.tick(), TICK_MS);
  }

  private stopClock() {
    if (this.clock !== null) clearInterval(this.clock);
    this.clock = null;
    if (this.flushTimer !== null) clearTimeout(this.flushTimer);
    this.flushTimer = null;
  }

  private tick() {
    const now = Date.now();
    this._now.set(new Date(now));
    for (const [key, entry] of this.entries) {
      if (
        entry.query.range.kind === 'live' &&
        alignDown(now, stepMs(entry.query.step)) !== entry.bucket
      ) {
        this.markDirty(key);
      }
    }
  }

  private markDirty(key: string) {
    this.dirty.add(key);
    this.flushTimer ??= setTimeout(() => this.flush(), FLUSH_DELAY_MS);
  }

  private flush() {
    this.flushTimer = null;
    const now = Date.now();
    const keys = [...this.dirty];
    this.dirty.clear();

    for (let i = 0; i < keys.length; i += MAX_BATCH_QUERIES) {
      const batch = keys.slice(i, i + MAX_BATCH_QUERIES).map(key => {
        const entry = this.entries.get(key)!;
        entry.bucket = alignDown(now, stepMs(entry.query.step));
        entry.seq++;
        return { key, seq: entry.seq, dto: toDto(entry.query, now) };
      });

      this.graphService.getBatchTimeseries(batch.map(b => b.dto)).subscribe({
        next: ({ results }) =>
          batch.forEach((b, idx) => this.apply(b.key, b.seq, results[idx])),
        error: (err: Error) =>
          batch.forEach(b =>
            this.update(b.key, b.seq, state => ({
              ...state,
              loading: false,
              error: err?.message ?? 'Failed to load timeseries',
            }))
          ),
      });
    }
  }

  private apply(
    key: string,
    seq: number,
    result: BatchTimeseriesResultDto | undefined
  ) {
    this.update(key, seq, state => {
      if (!result?.timeseries) {
        return {
          ...state,
          loading: false,
          error: result?.error ?? 'Missing timeseries in response',
        };
      }
      return {
        loading: false,
        error: null,
        unit: unitOf(result.timeseries),
        points: result.timeseries.map(t => [
          t.timestamp * 1000,
          toValue(t.value),
        ]),
      };
    });
  }

  private update(
    key: string,
    seq: number,
    fn: (state: SeriesState) => SeriesState
  ) {
    const entry = this.entries.get(key);
    if (entry && entry.seq === seq) entry.state.update(fn);
  }
}

function toDto(query: SeriesQuery, now: number): DeviceTimeseriesQueryDto {
  const { start, end } = resolveRange(query, now);
  return {
    device_id: query.device_id,
    sub_property: query.sub_property || undefined,
    step: query.step,
    start: start.toISOString(),
    end: end.toISOString(),
  };
}

function sameKeys(a: readonly SeriesQuery[], b: readonly SeriesQuery[]) {
  return (
    a.length === b.length && a.every((q, i) => queryKey(q) === queryKey(b[i]!))
  );
}

/**
 * Registers the given queries with the {@link TimeseriesStore} for the
 * lifetime of the calling component. Re-registers when the query keys change.
 * Must be called in an injection context.
 */
export function injectTimeseries(
  queries: () => readonly SeriesQuery[]
): Signal<SeriesResult[]> {
  const store = inject(TimeseriesStore);
  const current = computed(queries, { equal: sameKeys });
  const handles = signal<{ query: SeriesQuery; handle: SeriesHandle }[]>([]);

  let registered: { query: SeriesQuery; handle: SeriesHandle }[] = [];
  const releaseAll = (regs: typeof registered) =>
    regs.forEach(r => r.handle.release());

  // Register the new queries before releasing the old ones, so keys that
  // stay the same keep their cached data
  effect(() => {
    const qs = current();
    const previous = registered;
    registered = untracked(() =>
      qs.map(query => ({ query, handle: store.register(query) }))
    );
    releaseAll(previous);
    handles.set(registered);
  });
  inject(DestroyRef).onDestroy(() => releaseAll(registered));

  return computed(() =>
    handles().map(({ query, handle }) => ({ ...handle.state(), query }))
  );
}
