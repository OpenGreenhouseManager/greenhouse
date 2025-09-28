import { Component, computed, inject, input, signal } from '@angular/core';
import { AlertService } from './alert.service';
import { toSignal, toObservable } from '@angular/core/rxjs-interop';
import { catchError, map, switchMap } from 'rxjs/operators';
import { CardComponent } from '../../card/card.component';
import { combineLatest, of } from 'rxjs';
import { subHours } from 'date-fns';
import { Alert, Severity } from '../../alert/models/alert';
import { AlertDetailListComponent } from '../../alert/alert-detail-list/alert-detail-list.component';
import { AccordionModule } from 'primeng/accordion';

@Component({
  selector: 'grn-alert-list',
  standalone: true,
  imports: [CardComponent, AlertDetailListComponent, AccordionModule],
  templateUrl: './alert-list.component.html',
  styleUrl: './alert-list.component.scss',
})
export class AlertListComponent {
  public dataSourceId = input.required<string>();
  private alertService = inject(AlertService);

  public loading = signal(false);
  public error = signal<string | null>(null);

  public startDate = signal(subHours(new Date(), 1));

  public alertsSplitByIdentifier = computed(() => {
    const alerts: Map<string, Alert[]> = new Map();
    if (!this.alerts()) {
      return new Map();
    }
    for (const alert of this.alerts()!) {
      if (!alerts.has(alert.identifier)) {
        alerts.set(alert.identifier, []);
      }
      alerts.get(alert.identifier)!.push(alert);
    }
    return alerts;
  });

  // Convert the device signal to an observable, then switch to the timeseries observable
  public alerts = toSignal(
    combineLatest([
      toObservable(this.dataSourceId),
      toObservable(this.startDate),
    ])
      .pipe(
        switchMap(([deviceConfig, startDate]) => {
          this.loading.set(true);
          this.error.set(null);
          return this.alertService
            .getAlerts({
              datasource_id: deviceConfig,
              created_at: startDate,
            })
            .pipe(
              catchError(error => {
                this.loading.set(false);
                this.error.set(error.message);
                return of([]);
              })
            );
        })
      )
      .pipe(
        map(alerts => {
          this.loading.set(false);
          return alerts.map(
            a =>
              ({
                id: a.id,
                severity: this.convertSeverity(a.severity),
                identifier: a.identifier,
                value: a.value,
                note: a.note,
                created_at: new Date(a.created_at),
                datasource_id: a.datasource_id,
              }) satisfies Alert
          );
        })
      )
  );

  convertSeverity(severity: string): Severity {
    switch (severity) {
      case 'Info':
        return Severity.Info;
      case 'Warning':
        return Severity.Warning;
      case 'Error':
        return Severity.Error;
      case 'Fatal':
        return Severity.Fatal;
      default:
        throw new Error(`Unknown severity: ${severity}`);
    }
  }
}
