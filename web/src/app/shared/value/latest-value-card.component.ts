import { Component, computed, inject, input, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { catchError, map, of, switchMap, timer } from 'rxjs';
import { CardComponent } from '../../card/card.component';
import { GraphService } from '../graph/graph.service';

type LatestValue = {
  value: number | string | null;
  unit?: string;
};

@Component({
  selector: 'grn-latest-value-card',
  standalone: true,
  imports: [CardComponent],
  templateUrl: './latest-value-card.component.html',
  styleUrl: './latest-value-card.component.scss',
})
export class LatestValueCardComponent {
  public header = input.required<string>();
  public deviceId = input.required<string>();
  public subProperty = input<string>();
  private graphService = inject(GraphService);

  public loading = signal(false);
  public error = signal<string | null>(null);

  private pollMs = 30000;

  public latest = toSignal(
    timer(0, this.pollMs).pipe(
      switchMap(() => {
        this.loading.set(true);
        this.error.set(null);
        const end = new Date();
        const start = new Date(end.getTime() - 30000);
        return this.graphService
          .getTimeseries(this.deviceId(), {
            start,
            end,
            sub_property: this.subProperty() || undefined,
            step: '30s',
          })
          .pipe(
            map(res => {
              const last = (res?.timeseries ?? []).at(-1);
              if (!last) return { value: null } as LatestValue;
              const v: any = last.value;
              if (v && typeof v === 'object') {
                if ('Measurement' in v && v.Measurement) {
                  return {
                    value: v.Measurement.value as number,
                    unit: v.Measurement.unit as string,
                  } as LatestValue;
                }
                if ('Number' in v) {
                  return { value: v.Number as number } as LatestValue;
                }
                if ('Boolean' in v) {
                  return {
                    value: (v.Boolean ? 'true' : 'false') as string,
                  } as LatestValue;
                }
              }
              return { value: null } as LatestValue;
            }),
            catchError(err => {
              this.error.set(err?.message ?? 'Failed to load value');
              return of({ value: null } as LatestValue);
            })
          );
      })
    ),
    { initialValue: { value: null } as LatestValue }
  );

  public displayValue = computed(() => {
    const l = this.latest();
    if (l.value === null || l.value === undefined) return '—';
    if (typeof l.value === 'number') {
      const formatted = Number.isInteger(l.value)
        ? l.value.toString()
        : l.value.toFixed(1);
      return l.unit ? `${formatted}${l.unit}` : formatted;
    }
    return l.value.toString();
  });
}
