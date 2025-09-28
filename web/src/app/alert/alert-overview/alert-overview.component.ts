import { Component, effect, inject, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { endOfDay, startOfDay, subDays, subHours } from 'date-fns';
import { SortEvent } from 'primeng/api';
import { Button } from 'primeng/button';
import { Select } from 'primeng/select';
import { TableModule } from 'primeng/table';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { SeverityTagComponent } from '../../shared/tag/severity-tag.component';
import { AggregatedAlert } from '../models/aggregated-alert';
import { Severity } from '../models/alert';
import { AlertInterval } from '../models/alert-interval';
import { AlertService } from '../services/alert-service';

@Component({
  selector: 'grn-alert-overview',
  standalone: true,
  imports: [
    TableModule,
    FormsModule,
    Select,
    NavBarComponent,
    Button,
    SeverityTagComponent,
  ],
  templateUrl: './alert-overview.component.html',
  styleUrls: ['./alert-overview.component.scss'],
})
export class AlertOverviewComponent implements OnInit {
  private router = inject(Router);
  private alertService = inject(AlertService);

  Severity = Severity;

  alerts = signal<AggregatedAlert[]>([]);
  alerts_effect = effect(() => {
    this.alertService
      .getAlerts(
        this.intervalToDate(
          AlertInterval[
            this.selectedInterval().value as keyof typeof AlertInterval
          ] as AlertInterval
        ),
        endOfDay(new Date())
      )
      .subscribe(value => {
        this.alerts.set(value);
      });
  });
  intervals = Object.keys(AlertInterval)
    .filter(key => isNaN(Number(key)))
    .map(key => ({
      label: AlertInterval[key as keyof typeof AlertInterval],
      value: key,
    }));
  selectedInterval = signal(this.intervals[0]);
  selectedSeverity: Severity | null = null;
  severityOptions = Object.keys(Severity)
    .filter(key => isNaN(Number(key)))
    .map(key => ({
      label: Severity[key as keyof typeof Severity],
      value: key,
    }));

  ngOnInit() {}

  intervalToDate(interval: AlertInterval): Date {
    switch (interval) {
      case AlertInterval.last_1_hour:
        return subHours(new Date(), 1);
      case AlertInterval.last_24_hours:
        return subDays(new Date(), 1);
      case AlertInterval.last_7_days:
        return startOfDay(subDays(new Date(), 7));
      case AlertInterval.last_14_days:
        return startOfDay(subDays(new Date(), 14));
      case AlertInterval.last_30_days:
        return startOfDay(subDays(new Date(), 30));
      case AlertInterval.last_90_days:
        return startOfDay(subDays(new Date(), 90));
      case AlertInterval.absolute:
        return startOfDay(new Date());
    }
  }

  prettyDate(date: Date): string {
    return date.toLocaleDateString('de-DE', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  }

  display(identifier: string): string {
    const alias = this.searchForAlias(identifier);
    return (
      alias ||
      (identifier.length <= 16
        ? identifier
        : `${identifier.substring(0, 13)}...`)
    );
  }

  searchForAlias(identifier: string): string | null {
    return localStorage.getItem(`alias_${identifier}`);
  }

  editAlias(event: MouseEvent, identifier: string) {
    event.stopPropagation();
    const alias = this.searchForAlias(identifier);
    const newAlias = prompt('Edit Alias', alias || '');
    if (newAlias !== null) {
      if (newAlias.trim() === '') {
        localStorage.removeItem(`alias_${identifier}`);
      } else {
        localStorage.setItem(`alias_${identifier}`, newAlias);
      }
    }
  }

  customSort(event: SortEvent) {
    event.data!.sort((data1, data2) => {
      const value1 = data1[event.field!];
      const value2 = data2[event.field!];

      const result = value1 < value2 ? -1 : value1 > value2 ? 1 : 0;
      return event.order! * result;
    });
  }

  showDetails(alert: AggregatedAlert) {
    this.router.navigate(['/alert', alert.identifier, alert.source]);
  }
}
