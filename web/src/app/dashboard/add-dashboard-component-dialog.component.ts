import {
  Component,
  computed,
  inject,
  input,
  linkedSignal,
  output,
  signal,
} from '@angular/core';
import { rxResource } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { catchError, map, of } from 'rxjs';
import { ButtonDirective } from 'primeng/button';
import { Dialog } from 'primeng/dialog';
import { InputText } from 'primeng/inputtext';
import { Select } from 'primeng/select';
import { DeviceService } from '../device/services/device-service';
import type { DeviceResponseDto } from '../dtos/generated/device_service/DeviceResponseDto';
import { AlertFormComponent } from './alert-form.component';
import { MultiDataFormComponent } from './multi-data-form.component';
import { SingleDataFormComponent } from './single-data-form.component';

export interface DashboardComponentCreate {
  type: 'graph' | 'multiGraph' | 'alertList' | 'latestValue' | 'compass';
  graphData?: { deviceId: string; subProperty?: string }[];
  axisMode?: 'merged' | 'separate';
  dataSourceId?: string;
  name: string;
}

@Component({
  selector: 'grn-add-dashboard-component-dialog',
  imports: [
    Dialog,
    ButtonDirective,
    Select,
    InputText,
    FormsModule,
    SingleDataFormComponent,
    MultiDataFormComponent,
    AlertFormComponent,
  ],
  template: `
    <p-dialog
      header="Add Dashboard Component"
      [modal]="true"
      [visible]="visibleSignal()"
      class="w-full"
      [dismissableMask]="true"
      [closeOnEscape]="true"
      (onHide)="onCloseDialog()">
      <div class="flex items-center gap-4 mb-4 w-full">
        <label for="componentType" class="font-semibold w-24"
          >Component Type</label
        >
        <p-select
          id="componentType"
          [options]="[
            'graph',
            'multiGraph',
            'latestValue',
            'compass',
            'alertList',
          ]"
          [(ngModel)]="selectedComponentType"
          [appendTo]="'body'"
          class="w-full" />
      </div>
      <div class="flex items-center gap-4 mb-4">
        <label for="name" class="font-semibold w-24">Name</label>
        <input pInputText id="name" [(ngModel)]="name" />
      </div>

      @if (
        selectedComponentType() === 'graph' ||
        selectedComponentType() === 'latestValue' ||
        selectedComponentType() === 'compass'
      ) {
        <grn-single-data-form
          [devices]="devices()"
          (selectedDevice)="singleData.set($event)" />
      }

      @if (selectedComponentType() === 'multiGraph') {
        <grn-multi-data-form
          [devices]="devices()"
          (selectedDevices)="multiData.set($event)" />
      }

      @if (selectedComponentType() === 'alertList') {
        <grn-alert-form (selectedDataSource)="dataSourceId.set($event)" />
      }

      <div class="flex justify-end gap-2 mt-6">
        <button
          pButton
          type="button"
          severity="secondary"
          (click)="onCloseDialog()">
          Cancel
        </button>
        <button
          pButton
          type="button"
          [disabled]="!isFormValid()"
          (click)="onSave()">
          Add Component
        </button>
      </div>
    </p-dialog>
  `,
})
export class AddDashboardComponentDialogComponent {
  visible = input<boolean>(false);
  visibleSignal = linkedSignal(() => this.visible());
  dialogClose = output<void>();
  componentAdded = output<DashboardComponentCreate>();
  deviceService = inject(DeviceService);

  // Reset to defaults whenever the dialog is opened or closed
  selectedComponentType = linkedSignal<
    boolean,
    DashboardComponentCreate['type']
  >({
    source: this.visible,
    computation: () => 'graph',
  });
  name = linkedSignal({ source: this.visible, computation: () => '' });

  // Load all devices for autocomplete each time the dialog opens
  private devicesResource = rxResource({
    params: () => this.visible() || undefined,
    stream: () =>
      this.deviceService.getAllDevices().pipe(
        map(res => res.devices ?? []),
        catchError(() => of([]))
      ),
    defaultValue: [],
  });
  devices = computed<DeviceResponseDto[]>(() =>
    this.devicesResource.hasValue() ? this.devicesResource.value() : []
  );
  dataSourceId = signal<string | null>(null);

  singleData = signal<{
    deviceId: string;
    subProperty: string;
  } | null>(null);

  multiData = signal<{
    axisMode: 'merged' | 'separate';
    graphs: {
      deviceId: string;
      subProperty: string;
    }[];
  } | null>(null);

  isFormValid = computed(() => {
    const type = this.selectedComponentType();
    if (!type) return false;

    if (!this.name()) return false;

    if (type === 'graph' || type === 'latestValue' || type === 'compass') {
      return this.singleData() !== null;
    }

    if (type === 'multiGraph') {
      // Require at least 2 valid device IDs

      const validCount =
        this.multiData()?.graphs.filter(e => e.deviceId.trim() !== '').length ??
        0;
      return validCount >= 2;
    }

    if (type === 'alertList') {
      const data = this.dataSourceId();
      if (!data) {
        return false;
      }
      return data.trim() !== '';
    }
    return false;
  });

  onCloseDialog() {
    this.dialogClose.emit();
    this.visibleSignal.set(false);
  }

  onSave() {
    if (!this.isFormValid()) return;

    const type = this.selectedComponentType();
    const component: DashboardComponentCreate = {
      type,
      name: this.name().trim(),
    };

    if (type === 'graph' || type === 'latestValue' || type === 'compass') {
      const data = this.singleData();
      if (!data) {
        return;
      }
      component.graphData = [data];
    } else if (type === 'multiGraph') {
      const data = this.multiData();
      if (!data) {
        return;
      }
      component.axisMode = data.axisMode;
      component.graphData = data.graphs
        .filter(e => e.deviceId.trim() !== '')
        .map(e => ({
          deviceId: e.deviceId.trim(),
          subProperty: e.subProperty.trim(),
        }));
    } else if (type === 'alertList') {
      const data = this.dataSourceId();
      if (!data) {
        return;
      }
      component.dataSourceId = data.trim();
    }

    this.componentAdded.emit(component);
    this.onCloseDialog();
  }
}
