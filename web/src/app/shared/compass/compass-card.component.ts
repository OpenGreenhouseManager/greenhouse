import { Component, computed, inject, input, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { catchError, map, of, switchMap, timer } from 'rxjs';
import { CardComponent } from '../../card/card.component';
import { GraphService } from '../graph/graph.service';

@Component({
  selector: 'grn-compass-card',
  standalone: true,
  imports: [CardComponent],
  templateUrl: './compass-card.component.html',
  styleUrl: './compass-card.component.scss',
})
export class CompassCardComponent {
  public header = input.required<string>();
  public deviceId = input.required<string>();
  public subProperty = input<string>();
  private graphService = inject(GraphService);

  public loading = signal(false);
  public error = signal<string | null>(null);
  private pollMs = 30000;

  public angle = toSignal(
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
              if (!last) return null as number | null;
              const v = last.value;
              if (v && typeof v === 'object') {
                if ('Measurement' in v && v.Measurement) {
                  return normalizeAngle(v.Measurement.value as number);
                }
                if ('Number' in v) {
                  return normalizeAngle(v.Number as number);
                }
              }
              return null as number | null;
            }),
            catchError(err => {
              this.error.set(err?.message ?? 'Failed to load value');
              return of(null as number | null);
            })
          );
      })
    ),
    { initialValue: null as number | null }
  );

  public cardinal = computed(() => {
    const a = this.angle();
    if (a === null || a === undefined) return '—';
    const directions = [
      'N',
      'NNE',
      'NE',
      'ENE',
      'E',
      'ESE',
      'SE',
      'SSE',
      'S',
      'SSW',
      'SW',
      'WSW',
      'W',
      'WNW',
      'NW',
      'NNW',
    ];
    const deg = ((a % 360) + 360) % 360; // normalize
    const index = Math.round(deg / 22.5) % 16;
    return directions[index];
  });

  public svgRotation = computed(() => {
    const a = this.angle() ?? 0;
    return `rotate(${a} 50 50)`;
  });
}

function normalizeAngle(a: number): number {
  if (!isFinite(a)) return 0;
  let x = a % 360;
  if (x < 0) x += 360;
  return x;
}
