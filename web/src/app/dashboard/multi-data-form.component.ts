import { Component, input, linkedSignal, output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ButtonDirective } from 'primeng/button';
import { Select } from 'primeng/select';
import type { DeviceResponseDto } from '../dtos/generated/device_service/DeviceResponseDto';
import { SingleDataFormComponent } from './single-data-form.component';

type AxisMode = 'merged' | 'separate';
type SeriesData = {
  deviceId: string;
  subProperty: string;
};
type SeriesEntry = {
  id: string;
  /** Prefill of the row's form, fixed so typing does not reset it. */
  initial: SeriesData | null;
  data: SeriesData;
};

@Component({
  selector: 'grn-multi-data-form',
  imports: [Select, ButtonDirective, FormsModule, SingleDataFormComponent],
  template: `
    <div class="flex items-center gap-4 mb-4">
      <label for="axisMode" class="font-semibold w-24">Axis Mode</label>
      <p-select
        inputId="axisMode"
        [options]="axisModeOptions"
        [ngModel]="axisMode()"
        (ngModelChange)="onAxisModeChange($event)"
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
            [initial]="entry.initial"
            (selectedDevice)="updateForm(entry.id, $event)"
            class="w-full" />
          <button
            pButton
            type="button"
            severity="danger"
            iconOnly
            (click)="removeData(entry.id)"
            [disabled]="multiData().length <= 1">
            <i class="pi pi-trash"></i>
          </button>
        </div>
      }
    </div>
  `,
})
export class MultiDataFormComponent {
  devices = input.required<DeviceResponseDto[]>();
  /** Prefills the form, e.g. when editing an existing widget. */
  initial = input<{ axisMode: AxisMode; graphs: SeriesData[] } | null>(null);

  axisMode = linkedSignal<AxisMode>(() => this.initial()?.axisMode ?? 'merged');
  axisModeOptions: AxisMode[] = ['merged', 'separate'];

  selectedDevices = output<{
    axisMode: AxisMode;
    graphs: {
      deviceId: string;
      subProperty: string;
    }[];
  }>();

  multiData = linkedSignal<SeriesEntry[]>(() => [
    ...(this.initial()?.graphs ?? []).map(data => ({
      id: nextId(),
      initial: data,
      data,
    })),
    emptyEntry(),
  ]);

  onAxisModeChange(axisMode: AxisMode) {
    this.axisMode.set(axisMode);
    this.emitSelection();
  }

  updateForm(id: string, data: { deviceId: string; subProperty: string }) {
    this.setEntries(
      this.multiData().map(entry =>
        entry.id === id ? { ...entry, data } : entry
      )
    );
  }

  removeData(id: string) {
    this.setEntries(this.multiData().filter(entry => entry.id !== id));
  }

  // Keeps exactly one trailing empty row to type the next series into
  private setEntries(entries: SeriesEntry[]) {
    const hasEmptyRow = entries.some(
      entry => entry.data.deviceId === '' && entry.data.subProperty === ''
    );
    this.multiData.set(hasEmptyRow ? entries : [...entries, emptyEntry()]);
    this.emitSelection();
  }

  private emitSelection() {
    this.selectedDevices.emit({
      axisMode: this.axisMode(),
      graphs: this.multiData()
        .map(entry => entry.data)
        .filter(data => data.deviceId !== '' && data.subProperty !== ''),
    });
  }
}

// Row ids only need to be unique within the page. Not crypto.randomUUID():
// it is missing outside secure contexts, e.g. plain HTTP on a local network.
let lastId = 0;
function nextId(): string {
  return `series-${++lastId}`;
}

function emptyEntry(): SeriesEntry {
  return {
    id: nextId(),
    initial: null,
    data: { deviceId: '', subProperty: '' },
  };
}
