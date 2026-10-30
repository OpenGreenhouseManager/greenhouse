import { Component, computed, input } from '@angular/core';
import { Tag } from 'primeng/tag';
import { Severity } from '../../alert/models/alert';

@Component({
  selector: 'grn-severity-tag',
  imports: [Tag],
  template: `
    <p-tag
      [value]="severityString()"
      [style]="{ backgroundColor: severityBackground() }" />
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
        return '#4f8a9a';
      case Severity.Warning:
        return '#d99a2b';
      case Severity.Error:
        return '#c2483d';
      case Severity.Fatal:
        return '#8f2d26';
      default:
        return 'warn';
    }
  });
}
