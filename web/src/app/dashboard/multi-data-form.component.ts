import { Component, effect, input, output, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Button } from 'primeng/button';
import { Select } from 'primeng/select';
import { v4 as uuidv4 } from 'uuid';
import type { DeviceResponseDto } from '../dtos/device';
import { SingleDataFormComponent } from './single-data-form.component';

@Component({
  selector: 'grn-multi-data-form',
  standalone: true,
  imports: [Select, Button, FormsModule, SingleDataFormComponent],
  template: `
    <div class="flex items-center gap-4 mb-4">
      <label class="font-semibold w-24">Axis Mode</label>
      <p-select
        [options]="axisModeOptions"
        [(ngModel)]="axisMode"
        [appendTo]="'body'"
        class="w-full" />
    </div>
    <div class="mb-2 font-semibold">Series</div>
    <div class="flex flex-col gap-3">
      @for (entry of multiData(); track entry.id) {
        <div class="flex items-center gap-2">
          <grn-single-data-form
            [devices]="devices()"
            [compact]="true"
            (selectedDevice)="updateForm(entry.id, $event)"
            class="w-full" />
          <p-button
            icon="pi pi-trash"
            severity="danger"
            (click)="removeData(entry.id)"
            [disabled]="multiData().length <= 1" />
        </div>
      }
    </div>
  `,
})
export class MultiDataFormComponent {
  axisMode: 'merged' | 'separate' = 'merged';
  axisModeOptions: ('merged' | 'separate')[] = ['merged', 'separate'];

  devices = input.required<DeviceResponseDto[]>();

  selectedDevices = output<{
    axisMode: 'merged' | 'separate';
    graphs: {
      deviceId: string;
      subProperty: string;
    }[];
  }>();

  multiData = signal<
    {
      id: string;
      data: {
        deviceId: string;
        subProperty: string;
      };
    }[]
  >([{ id: uuidv4(), data: { deviceId: '', subProperty: '' } }]);

  constructor() {
    effect(() => {
      const data = this.multiData();
      if (!data) {
        return;
      }
      this.selectedDevices.emit({
        axisMode: this.axisMode,
        graphs: data
          .map(entry => entry.data)
          .filter(data => data.deviceId !== '' && data.subProperty !== ''),
      });
      if (
        data
          .map(entry => entry.data)
          .filter(data => data.deviceId === '' && data.subProperty === '')
          .length === 0
      ) {
        this.addData();
      }
    });
  }

  updateForm(id: string, data: { deviceId: string; subProperty: string }) {
    this.multiData.update(prev =>
      prev.map(entry => (entry.id === id ? { ...entry, data } : entry))
    );
  }

  removeData(id: string) {
    this.multiData.update(prev => prev.filter(entry => entry.id !== id));
  }

  addData() {
    this.multiData.update(prev => [
      ...prev,
      { id: uuidv4(), data: { deviceId: '', subProperty: '' } },
    ]);
  }
}
