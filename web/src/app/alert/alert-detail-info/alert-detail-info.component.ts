import { isPlatformBrowser } from '@angular/common';
import {
  ChangeDetectorRef,
  Component,
  computed,
  inject,
  input,
  OnInit,
  PLATFORM_ID,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import {
  addDays,
  endOfDay,
  formatDistance,
  isSameDay,
  startOfDay,
  subDays,
} from 'date-fns';
import { ButtonModule } from 'primeng/button';
import { CardModule } from 'primeng/card';
import { ChartModule } from 'primeng/chart';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { InputTextModule } from 'primeng/inputtext';
import { CardComponent } from '../../card/card.component';
import { barChartOptions, chartColors } from '../../shared/graph/chart_option';
import { Alert, Severity } from '../models/alert';

@Component({
  selector: 'grn-alert-detail-info',
  standalone: true,
  imports: [
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    InputGroupAddonModule,
    CardModule,
    ChartModule,
    CardComponent,
  ],
  templateUrl: './alert-detail-info.component.html',
  styleUrl: './alert-detail-info.component.scss',
})
export class AlertDetailInfoComponent implements OnInit {
  private cd = inject(ChangeDetectorRef);

  data: unknown;
  options = barChartOptions;
  platformId = inject(PLATFORM_ID);

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

  ngOnInit() {
    this.initChart();
  }

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

  initChart() {
    if (isPlatformBrowser(this.platformId)) {
      const days = 8;

      this.data = {
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

      this.cd.markForCheck();
    }
  }
}
