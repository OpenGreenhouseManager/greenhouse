import {
  Component,
  effect,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Button } from 'primeng/button';
import { Dialog } from 'primeng/dialog';
import { InputText } from 'primeng/inputtext';
import { Select } from 'primeng/select';
import { DeviceService } from '../device/services/device-service';
import type { DeviceResponseDto } from '../dtos/device';
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
  standalone: true,
  imports: [
    Dialog,
    Button,
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
        selectedComponentType === 'graph' ||
        selectedComponentType === 'latestValue' ||
        selectedComponentType === 'compass'
      ) {
        <grn-single-data-form
          [devices]="devices()"
          (selectedDevice)="singleData.set($event)" />
      }

      @if (selectedComponentType === 'multiGraph') {
        <grn-multi-data-form
          [devices]="devices()"
          (selectedDevices)="multiData.set($event)" />
      }

      @if (selectedComponentType === 'alertList') {
        <grn-alert-form (selectedDataSource)="dataSourceId.set($event)" />
      }

      <div class="flex justify-end gap-2 mt-6">
        <p-button
          label="Cancel"
          severity="secondary"
          (click)="onCloseDialog()" />
        <p-button
          label="Add Component"
          [disabled]="!isFormValid()"
          (click)="onSave()" />
      </div>
    </p-dialog>
  `,
})
export class AddDashboardComponentDialogComponent {
  visible = input<boolean>(false);
  visibleSignal = signal(this.visible());
  dialogClose = output<void>();
  componentAdded = output<DashboardComponentCreate>();
  deviceService = inject(DeviceService);

  selectedComponentType:
    | 'graph'
    | 'multiGraph'
    | 'alertList'
    | 'latestValue'
    | 'compass' = 'graph';
  devices = signal<DeviceResponseDto[]>([]);
  dataSourceId = signal<string | null>(null);
  name = '';

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

  constructor() {
    effect(() => {
      this.visibleSignal.set(this.visible());
      if (this.visible()) {
        this.resetForm();
        this.loadAllDevices();
      }
    });
  }

  resetForm() {
    this.selectedComponentType = 'graph';
    this.name = '';
  }

  isFormValid(): boolean {
    if (!this.selectedComponentType) return false;

    if (!this.name) return false;

    if (
      this.selectedComponentType === 'graph' ||
      this.selectedComponentType === 'latestValue' ||
      this.selectedComponentType === 'compass'
    ) {
      return this.singleData() !== null;
    }

    if (this.selectedComponentType === 'multiGraph') {
      // Require at least 2 valid device IDs

      const validCount =
        this.multiData()?.graphs.filter(e => e.deviceId.trim() !== '').length ??
        0;
      return validCount >= 2;
    }

    if (this.selectedComponentType === 'alertList') {
      const data = this.dataSourceId();
      if (!data) {
        return false;
      }
      return data.trim() !== '';
    }
    return false;
  }

  onCloseDialog() {
    this.dialogClose.emit();
    this.visibleSignal.set(false);
  }

  // Load all devices once for autocomplete
  loadAllDevices() {
    this.deviceService.getAllDevices().subscribe({
      next: res => {
        this.devices.set(res.devices ?? []);
      },
      error: () => {
        this.devices.set([]);
      },
    });
  }

  onSave() {
    if (!this.isFormValid()) return;

    const component: DashboardComponentCreate = {
      type: this.selectedComponentType,
      name: this.name.trim(),
    };

    if (
      this.selectedComponentType === 'graph' ||
      this.selectedComponentType === 'latestValue' ||
      this.selectedComponentType === 'compass'
    ) {
      const data = this.singleData();
      if (!data) {
        return;
      }
      component.graphData = [data];
    } else if (this.selectedComponentType === 'multiGraph') {
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
    } else if (this.selectedComponentType === 'alertList') {
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
