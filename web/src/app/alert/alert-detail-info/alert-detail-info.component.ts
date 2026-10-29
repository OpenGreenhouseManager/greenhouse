import { Component, computed, input } from '@angular/core';
import type { ChartData } from 'chart.js';
import {
  addDays,
  endOfDay,
  formatDistance,
  isSameDay,
  startOfDay,
  subDays,
} from 'date-fns';
import { ChartModule } from 'primeng/chart';
import { CardComponent } from '../../card/card.component';
import { barChartOptions, chartColors } from '../../shared/graph/chart_option';
import type { Alert } from '../models/alert';
import { Severity } from '../models/alert';

@Component({
  selector: 'grn-alert-detail-info',
  imports: [ChartModule, CardComponent],
  templateUrl: './alert-detail-info.component.html',
  styleUrl: './alert-detail-info.component.scss',
})
export class AlertDetailInfoComponent {
  options = barChartOptions;

  alerts = input.required<Alert[]>();

  lastSeen = computed(() => {
    if (this.alerts().length === 0) {
      return 'No alerts';
    }
    const lastAlert = this.alerts().reduce((latest, current) => {
      const currentDate = new Date(current.created_at);
      const latestDate = new Date(latest.created_at);
      return currentDate > latestDate ? current : latest;
    });
    return formatDistance(lastAlert.created_at, new Date());
  });

  firstSeen = computed(() => {
    if (this.alerts().length === 0) {
      return 'No alerts';
    }
    const firstAlert = this.alerts().reduce((earliest, current) => {
      const currentDate = new Date(current.created_at);
      const earliestDate = new Date(earliest.created_at);
      return currentDate < earliestDate ? current : earliest;
    });
    return formatDistance(firstAlert.created_at, new Date());
  });

  getCount(severity: Severity, days: number): number[] {
    const data = [];
    for (
      let currentDay = startOfDay(subDays(Date.now(), days - 1));
      !isSameDay(currentDay, endOfDay(addDays(Date.now(), 1)));
      currentDay.setDate(currentDay.getDate() + 1)
    ) {
      const count = this.alerts().filter(
        alert =>
          alert.severity === severity &&
          isSameDay(new Date(alert.created_at), currentDay)
      ).length;
      data.push(count);
    }
    return data;
  }

  relativeDayNames(days: number): string[] {
    const names = [];
    for (let i = 0; i < days; i++) {
      const date = subDays(new Date(), days - i - 1);
      names.push(date.toLocaleDateString('en-US', { weekday: 'short' }));
    }
    return names;
  }

  data = computed<ChartData<'bar'>>(() => {
    const days = 8;

    return {
      labels: this.relativeDayNames(days),
      datasets: [
        {
          type: 'bar',
          label: 'Info',
          backgroundColor: chartColors.info,
          data: this.getCount(Severity.Info, days),
        },
        {
          type: 'bar',
          label: 'Warning',
          backgroundColor: chartColors.warning,
          data: this.getCount(Severity.Warning, days),
        },
        {
          type: 'bar',
          label: 'Error',
          backgroundColor: chartColors.error,
          data: this.getCount(Severity.Error, days),
        },
        {
          type: 'bar',
          label: 'Fatal',
          backgroundColor: chartColors.fatal,
          data: this.getCount(Severity.Fatal, days),
        },
      ],
    };
  });
}
