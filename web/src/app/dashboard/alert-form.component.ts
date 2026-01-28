import { Component, effect, output, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { InputText } from 'primeng/inputtext';

@Component({
  selector: 'grn-alert-form',
  standalone: true,
  imports: [InputText, FormsModule],
  template: `
    <div class="flex items-center gap-4 mb-4">
      <label for="dataSourceId" class="font-semibold w-24"
        >Data Source ID</label
      >
      <input pInputText id="dataSourceId" [(ngModel)]="dataSourceId" />
    </div>
  `,
})
export class AlertFormComponent {
  dataSourceId = signal<string>('');
  selectedDataSource = output<string>();

  constructor() {
    effect(() => {
      if (this.dataSourceId()) {
        this.selectedDataSource.emit(this.dataSourceId());
      }
    });
  }
}
