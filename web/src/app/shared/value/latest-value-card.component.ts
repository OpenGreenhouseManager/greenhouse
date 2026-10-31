import { Component, computed, input } from '@angular/core';
import { CardComponent } from '../../card/card.component';
import { injectTimeseries } from '../timeseries/timeseries.store';

@Component({
  selector: 'grn-latest-value-card',
  imports: [CardComponent],
  templateUrl: './latest-value-card.component.html',
  styleUrl: './latest-value-card.component.scss',
})
export class LatestValueCardComponent {
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

  private state = computed(() => this.series()[0]);

  public error = computed(() => this.state()?.error ?? null);

  public displayValue = computed(() => {
    const state = this.state();
    const value = state?.points.at(-1)?.[1];
    if (value === null || value === undefined) return '—';
    if (typeof value === 'number') {
      const formatted = Number.isInteger(value)
        ? value.toString()
        : value.toFixed(1);
      return state?.unit ? `${formatted}${state.unit}` : formatted;
    }
    return value.toString();
  });
}
