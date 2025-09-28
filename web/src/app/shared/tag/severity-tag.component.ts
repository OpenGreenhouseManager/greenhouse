import { Component, computed, input } from '@angular/core';
import { Tag } from 'primeng/tag';
import { Severity } from '../../alert/models/alert';

@Component({
  selector: 'grn-severity-tag',
  imports: [Tag],
  template: `
    <p-tag
      [value]="severityString()"
      [style]="{ backgroundColor: severityBackground() }"></p-tag>
  `,
})
export class SeverityTagComponent {
  severity = input.required<Severity>();

  severityString = computed(() => {
    return Severity[this.severity()];
  });

  severityBackground = computed(() => {
    switch (this.severity()) {
      case Severity.Info:
        // bluish white
        return '#66A9A9';
      case Severity.Warning:
        return '#f59e0b';
      case Severity.Error:
        return '#f44336';
      case Severity.Fatal:
        return '#b71c1c';
      default:
        return 'warn';
    }
  });
}
