import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { Alert, Severity } from '../models/alert';
import { TableModule } from 'primeng/table';
import { SortEvent } from 'primeng/api';
import { SelectModule } from 'primeng/select';
import { TagModule } from 'primeng/tag';

@Component({
  selector: 'grn-alert-detail-list',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    InputGroupAddonModule,
    TableModule,
    SelectModule,
    TagModule,
  ],
  templateUrl: './alert-detail-list.component.html',
  styleUrl: './alert-detail-list.component.scss',
})
export class AlertDetailListComponent {
  Severity = Severity;
  alerts = input.required<Alert[]>();

  selectedSeverity: Severity | null = null;
  severityOptions = Object.keys(Severity)
    .filter(key => isNaN(Number(key)))
    .map(key => ({
      label: Severity[key as keyof typeof Severity],
      value: key,
    }));

  customSort(event: SortEvent) {
    event.data!.sort((data1, data2) => {
      const value1 = data1[event.field!];
      const value2 = data2[event.field!];

      const result = value1 < value2 ? -1 : value1 > value2 ? 1 : 0;
      return event.order! * result;
    });
  }

  severityToColor(severity: Severity): string {
    switch (severity) {
      case Severity.Info:
        return 'info';
      case Severity.Warning:
        return 'warn';
      case Severity.Error:
        return 'error';
      case Severity.Fatal:
        return 'fatal';
      default:
        return 'gray';
    }
  }

  prittyDate(date: Date): string {
    return date.toLocaleDateString('de-DE', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  }
}
