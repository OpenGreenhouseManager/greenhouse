import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import {
  HttpTestingController,
  provideHttpClientTesting,
} from '@angular/common/http/testing';
import { GraphComponent } from './graph.component';
import { provideEchartsCore } from 'ngx-echarts';
import { echarts } from './echarts';
import type { BatchTimeseriesRequestDto } from '../../dtos/generated/device_service/BatchTimeseriesRequestDto';
import { FLUSH_DELAY_MS, TICK_MS } from '../timeseries/timeseries.store';

describe('GraphComponent', () => {
  let component: GraphComponent;
  let fixture: ComponentFixture<GraphComponent>;
  let http: HttpTestingController;

  beforeEach(async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:01:00Z'));
    await TestBed.configureTestingModule({
      imports: [GraphComponent],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting(),
        provideEchartsCore({ echarts }),
      ],
    }).compileComponents();

    http = TestBed.inject(HttpTestingController);
    fixture = TestBed.createComponent(GraphComponent);
    component = fixture.componentInstance;
    fixture.componentRef.setInput('config', {
      graph_data: [
        {
          device_id: 'test-device-id',
          sub_property: 'test-property',
        },
        {
          device_id: 'other-device-id',
        },
      ],
    });
    fixture.componentRef.setInput('header', 'Test Graph Header');
    fixture.detectChanges();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('fetches all series in one batch request', () => {
    vi.advanceTimersByTime(FLUSH_DELAY_MS);

    const req = http.expectOne(r => r.url.endsWith('/timeseries/batch'));
    const { queries } = req.request.body as BatchTimeseriesRequestDto;
    expect(queries.map(q => q.device_id)).toEqual([
      'test-device-id',
      'other-device-id',
    ]);
    req.flush({
      results: [
        {
          query: queries[0],
          timeseries: [{ timestamp: 1, value: { Number: 2 } }],
        },
        { query: queries[1], error: 'boom' },
      ],
    });

    expect(component.timeseries()[0]?.data).toEqual([[1000, 2]]);
    expect(component.error()).toBe('other-device-id: boom');
    http.verify();
  });

  describe('auto-live range', () => {
    const batches = () => http.match(r => r.url.endsWith('/timeseries/batch'));
    const flushAll = () => batches().forEach(req => req.flush({ results: [] }));

    beforeEach(() => {
      vi.advanceTimersByTime(FLUSH_DELAY_MS);
      flushAll();
    });

    it('freezes the graph when the end is in the past', () => {
      component.setEnd(new Date(Date.now() - 2 * 3600_000));
      fixture.detectChanges();
      expect(component.range().kind).toBe('fixed');
      vi.advanceTimersByTime(FLUSH_DELAY_MS);
      flushAll();

      vi.advanceTimersByTime(TICK_MS * 20);
      expect(batches()).toHaveLength(0);
    });

    it('keeps refreshing when the end is now', () => {
      component.setEnd(new Date());
      fixture.detectChanges();
      expect(component.range().kind).toBe('live');

      // 24h window -> 5m step, refreshed once the step bucket changes
      vi.advanceTimersByTime(5 * 60_000 + FLUSH_DELAY_MS);
      expect(batches().length).toBeGreaterThan(0);
      flushAll();
    });
  });
});
