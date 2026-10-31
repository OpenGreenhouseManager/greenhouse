import { Component, computed, input } from '@angular/core';
import { CardComponent } from '../../card/card.component';
import { injectTimeseries } from '../timeseries/timeseries.store';

@Component({
  selector: 'grn-compass-card',
  imports: [CardComponent],
  templateUrl: './compass-card.component.html',
  styleUrl: './compass-card.component.scss',
})
export class CompassCardComponent {
  public header = input.required<string>();
  public deviceId = input.required<string>();
  public subProperty = input<string>();

  private series = injectTimeseries(() => [
    {
      device_id: this.deviceId(),
      sub_property: this.subProperty(),
      step: '15s',
      range: { kind: 'live', windowMs: 60_000 },
    },
  ]);

  public error = computed(() => this.series()[0]?.error ?? null);

  public angle = computed(() => {
    const value = this.series()[0]?.points.at(-1)?.[1];
    return typeof value === 'number' ? normalizeAngle(value) : null;
  });
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
