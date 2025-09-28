import {
  Component,
  computed,
  effect,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { toObservable, toSignal } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { Button } from 'primeng/button';
import { Dialog } from 'primeng/dialog';
import { InputText } from 'primeng/inputtext';
import { Select } from 'primeng/select';
import { catchError, of, switchMap } from 'rxjs';
import { DeviceService } from '../device/services/device-service';

export interface DashboardComponentCreate {
  type: 'graph' | 'alertList';
  deviceId?: string;
  subProperty?: string;
  dataSourceId?: string;
  name: string;
}

@Component({
  selector: 'grn-add-dashboard-component-dialog',
  standalone: true,
  imports: [Dialog, Button, Select, InputText, FormsModule],
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
          [options]="['graph', 'alertList']"
          [(ngModel)]="selectedComponentType"
          [appendTo]="'body'"
          class="w-full" />
      </div>
      <div class="flex items-center gap-4 mb-4">
        <label for="name" class="font-semibold w-24">Name</label>
        <input pInputText id="name" [(ngModel)]="name" />
      </div>

      @if (selectedComponentType === 'graph') {
        <div class="flex items-center gap-4 mb-4">
          <label for="deviceId" class="font-semibold w-24">Device ID</label>
          <input
            pInputText
            id="deviceId"
            type="text"
            [(ngModel)]="deviceIdInput"
            placeholder="Enter UUID" />
        </div>

        <div class="flex items-center gap-4 mb-4">
          <label for="subProperty" class="font-semibold w-24"
            >Sub Property</label
          >
          <p-select
            id="subProperty"
            [options]="subPropertyOptions()"
            [(ngModel)]="subProperty"
            [appendTo]="'body'"
            [disabled]="!validDeviceId() || subPropertyOptions()?.length === 0"
            class="w-full" />
        </div>
      }

      @if (selectedComponentType === 'alertList') {
        <div class="flex items-center gap-4 mb-4">
          <label for="dataSourceId" class="font-semibold w-24"
            >Data Source ID</label
          >
          <input pInputText id="dataSourceId" [(ngModel)]="dataSourceId" />
        </div>
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

  selectedComponentType: 'graph' | 'alertList' = 'graph';
  deviceIdInput = signal<string>('');
  subProperty = '';
  dataSourceId = '';
  name = '';

  // UUID validation regex - matches standard UUID format
  private uuidRegex =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

  // Computed signal for valid device ID
  validDeviceId = computed(() => {
    const input = this.deviceIdInput().trim();
    return input !== '' && this.uuidRegex.test(input);
  });

  // Computed signal for the validated device ID (empty if invalid)
  deviceId = computed(() => {
    return this.validDeviceId() ? this.deviceIdInput().trim() : '';
  });

  subPropertyOptions = toSignal(
    toObservable(this.deviceId).pipe(
      switchMap(deviceId => {
        if (!deviceId) return of([]);
        console.log('deviceId', deviceId);
        return this.deviceService.getDeviceOptions(deviceId).pipe(
          catchError(error => {
            console.error('Error fetching device options:', error);
            return of([]);
          })
        );
      })
    )
  );

  constructor() {
    effect(() => {
      this.visibleSignal.set(this.visible());
      if (this.visible()) {
        this.resetForm();
      }
    });
  }

  resetForm() {
    this.selectedComponentType = 'graph';
    this.deviceIdInput.set('');
    this.subProperty = '';
    this.dataSourceId = '';
    this.name = '';
  }

  isFormValid(): boolean {
    if (!this.selectedComponentType) return false;

    if (!this.name) return false;

    if (this.selectedComponentType === 'graph') {
      return this.validDeviceId();
    }

    if (this.selectedComponentType === 'alertList') {
      return this.dataSourceId.trim() !== '';
    }

    return false;
  }

  onCloseDialog() {
    this.dialogClose.emit();
    this.visibleSignal.set(false);
  }

  onSave() {
    if (!this.isFormValid()) return;

    const component: DashboardComponentCreate = {
      type: this.selectedComponentType!,
      name: this.name.trim(),
    };

    if (this.selectedComponentType === 'graph') {
      component.deviceId = this.deviceId();
      component.subProperty = this.subProperty.trim();
    } else if (this.selectedComponentType === 'alertList') {
      component.dataSourceId = this.dataSourceId.trim();
    }

    this.componentAdded.emit(component);
    this.onCloseDialog();
  }
}
