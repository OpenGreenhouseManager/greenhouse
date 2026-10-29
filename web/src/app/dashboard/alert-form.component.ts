import { Component, output, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { InputText } from 'primeng/inputtext';

@Component({
  selector: 'grn-alert-form',
  imports: [InputText, FormsModule],
  template: `
    <div class="flex items-center gap-4 mb-4">
      <label for="dataSourceId" class="font-semibold w-24"
        >Data Source ID</label
      >
      <input
        pInputText
        id="dataSourceId"
        [ngModel]="dataSourceId()"
        (ngModelChange)="onDataSourceIdChange($event)" />
    </div>
  `,
})
export class AlertFormComponent {
  dataSourceId = signal<string>('');
  selectedDataSource = output<string>();

  onDataSourceIdChange(dataSourceId: string) {
    this.dataSourceId.set(dataSourceId);
    // also emit an empty value, so clearing the field invalidates the form
    this.selectedDataSource.emit(dataSourceId);
  }
}
