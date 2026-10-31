import { provideHttpClient } from '@angular/common/http';
import {
  HttpTestingController,
  provideHttpClientTesting,
} from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import type { BatchTimeseriesRequestDto } from '../../dtos/generated/device_service/BatchTimeseriesRequestDto';
import type { SeriesQuery } from './timeseries.store';
import {
  FLUSH_DELAY_MS,
  pickStep,
  TICK_MS,
  TimeseriesStore,
} from './timeseries.store';

const live = (device_id: string, windowMs = 60_000): SeriesQuery => ({
  device_id,
  step: '15s',
  range: { kind: 'live', windowMs },
});

describe('pickStep', () => {
  it('picks the smallest rung covering the range', () => {
    expect(pickStep(60_000, 100)).toBe('5s');
    expect(pickStep(24 * 3600_000, 500)).toBe('5m');
    expect(pickStep(365 * 24 * 3600_000, 100)).toBe('1d');
  });
});

describe('TimeseriesStore', () => {
  let store: TimeseriesStore;
  let http: HttpTestingController;

  const expectBatch = () => {
    const req = http.expectOne(r => r.url.endsWith('/timeseries/batch'));
    return req;
  };
  const queriesOf = (req: ReturnType<typeof expectBatch>) =>
    (req.request.body as BatchTimeseriesRequestDto).queries;

  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    store = TestBed.inject(TimeseriesStore);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    http.verify();
    vi.useRealTimers();
  });

  it('dedupes identical queries into one batched query', () => {
    const a = store.register(live('dev-1'));
    const b = store.register(live('dev-1'));
    vi.advanceTimersByTime(FLUSH_DELAY_MS);

    const req = expectBatch();
    expect(queriesOf(req)).toHaveLength(1);
    req.flush({
      results: [
        {
          query: queriesOf(req)[0],
          timeseries: [
            { timestamp: 10, value: { Measurement: { value: 3, unit: '°C' } } },
          ],
        },
      ],
    });

    expect(a.state()).toEqual({
      loading: false,
      error: null,
      unit: '°C',
      points: [[10_000, 3]],
    });
    expect(b.state()).toBe(a.state());
  });

  it('batches different queries and chunks above the batch limit', () => {
    for (let i = 0; i < 51; i++) store.register(live(`dev-${i}`));
    vi.advanceTimersByTime(FLUSH_DELAY_MS);

    const reqs = http.match(r => r.url.endsWith('/timeseries/batch'));
    expect(reqs.map(r => queriesOf(r).length)).toEqual([50, 1]);
    reqs.forEach(r => r.flush({ results: [] }));
  });

  it('keeps per-query errors on their own series', () => {
    const ok = store.register(live('ok'));
    const bad = store.register(live('bad'));
    vi.advanceTimersByTime(FLUSH_DELAY_MS);

    const req = expectBatch();
    const [q1, q2] = queriesOf(req);
    req.flush({
      results: [
        { query: q1, timeseries: [{ timestamp: 1, value: { Number: 1 } }] },
        { query: q2, error: 'boom' },
      ],
    });

    expect(ok.state().error).toBeNull();
    expect(bad.state().error).toBe('boom');
    expect(bad.state().loading).toBe(false);
  });

  it('refreshes live queries on tick and stops once released', () => {
    const handle = store.register(live('dev-1'));
    vi.advanceTimersByTime(FLUSH_DELAY_MS);
    expectBatch().flush({ results: [] });

    vi.advanceTimersByTime(TICK_MS + FLUSH_DELAY_MS);
    expectBatch().flush({ results: [] });

    handle.release();
    vi.advanceTimersByTime(TICK_MS * 3);
    http.expectNone(r => r.url.endsWith('/timeseries/batch'));
  });

  it('only refreshes live queries when their step bucket changes', () => {
    store.register({ ...live('dev-1', 24 * 3600_000), step: '1h' });
    vi.advanceTimersByTime(FLUSH_DELAY_MS);
    expectBatch().flush({ results: [] });

    vi.advanceTimersByTime(TICK_MS);
    http.expectNone(r => r.url.endsWith('/timeseries/batch'));
  });

  it('does not refresh fixed ranges', () => {
    store.register({
      device_id: 'dev-1',
      step: '15s',
      range: { kind: 'fixed', start: new Date(0), end: new Date(60_000) },
    });
    vi.advanceTimersByTime(FLUSH_DELAY_MS);
    expectBatch().flush({ results: [] });

    vi.advanceTimersByTime(TICK_MS * 3);
    http.expectNone(r => r.url.endsWith('/timeseries/batch'));
  });
});
