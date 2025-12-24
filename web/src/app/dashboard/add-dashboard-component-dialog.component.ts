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
import { AutoComplete } from 'primeng/autocomplete';
import { Button } from 'primeng/button';
import { Dialog } from 'primeng/dialog';
import { InputText } from 'primeng/inputtext';
import { Select } from 'primeng/select';
import { catchError, of, switchMap } from 'rxjs';
import { DeviceService } from '../device/services/device-service';
import { DeviceResponseDto } from '../dtos/device';

export interface DashboardComponentCreate {
  type: 'graph' | 'multiGraph' | 'alertList';
  deviceId?: string;
  subProperty?: string;
  graphData?: { deviceId: string; subProperty?: string }[];
  axisMode?: 'merged' | 'separate';
  dataSourceId?: string;
  name: string;
}

@Component({
  selector: 'grn-add-dashboard-component-dialog',
  standalone: true,
  imports: [Dialog, Button, Select, InputText, AutoComplete, FormsModule],
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
          [options]="['graph', 'multiGraph', 'alertList']"
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
          <label for="deviceId" class="font-semibold w-24">Device</label>
          <p-autocomplete
            id="deviceId"
            class="w-full"
            [suggestions]="deviceSuggestions()"
            (completeMethod)="onDeviceQuery($event)"
            [(ngModel)]="deviceSearch"
            [optionLabel]="'name'"
            [dropdown]="true"
            [appendTo]="'body'"
            (onSelect)="onDeviceSelect($event)"
            (ngModelChange)="onDeviceSearchChange($event)" />
        </div>

        <div class="flex items-center gap-4 mb-4">
          <label for="subProperty" class="font-semibold w-24"
            >Sub Property</label
          >
          <p-select
            id="subProperty"
            [options]="subPropertyOptions().operations"
            [(ngModel)]="subProperty"
            [appendTo]="'body'"
            [disabled]="
              !validDeviceId() || subPropertyOptions().operations.length === 0
            "
            class="w-full" />
        </div>
      }

      @if (selectedComponentType === 'multiGraph') {
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
          @for (entry of multiEntries; track entry; let i = $index) {
            <div class="flex items-center gap-2">
              <p-autocomplete
                [id]="'deviceId_' + i"
                class="flex-1"
                [suggestions]="entry.suggestions"
                (completeMethod)="onEntryDeviceQuery(i, $event)"
                [(ngModel)]="entry.deviceSearch"
                [optionLabel]="'name'"
                [dropdown]="true"
                [appendTo]="'body'"
                (onSelect)="onEntryDeviceSelect(i, $event)"
                (ngModelChange)="onEntryDeviceSearchChange(i, $event)"
                placeholder="Search device by name or UUID" />
              <p-select
                [id]="'subProperty_' + i"
                class="flex-1"
                [options]="entry.options"
                [(ngModel)]="entry.subProperty"
                [appendTo]="'body'"
                [disabled]="
                  entry.deviceId.trim() === '' || entry.options.length === 0
                "
                placeholder="Select Sub Property" />
              <p-button
                icon="pi pi-trash"
                severity="danger"
                (click)="removeEntry(i)"
                [disabled]="multiEntries.length <= 1" />
            </div>
          }
          <div>
            <p-button
              label="Add Series"
              icon="pi pi-plus"
              (click)="addEntry()" />
          </div>
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

  selectedComponentType: 'graph' | 'multiGraph' | 'alertList' = 'graph';
  deviceIdInput = signal<string>('');
  deviceSearch = '';
  devices = signal<DeviceResponseDto[]>([]);
  deviceSuggestions = signal<DeviceResponseDto[]>([]);
  subProperty = '';
  dataSourceId = '';
  name = '';
  axisMode: 'merged' | 'separate' = 'merged';
  axisModeOptions: ('merged' | 'separate')[] = ['merged', 'separate'];
  multiEntries: {
    deviceId: string;
    deviceSearch: string;
    subProperty: string;
    options: string[];
    suggestions: DeviceResponseDto[];
  }[] = [
    {
      deviceId: '',
      deviceSearch: '',
      subProperty: '',
      options: [],
      suggestions: [],
    },
  ];

  // Consider any non-empty string a valid device id (IDs may not be UUIDs)
  validDeviceId = computed(() => {
    const input = this.deviceIdInput().trim();
    return input !== '';
  });

  // Always the trimmed device id (may be any non-empty string)
  deviceId = computed(() => this.deviceIdInput().trim());

  subPropertyOptions = toSignal(
    toObservable(this.deviceId).pipe(
      switchMap(deviceId => {
        if (!deviceId) return of({ operations: [] });
        console.log('deviceId', deviceId);
        return this.deviceService.getDeviceOptions(deviceId).pipe(
          catchError(error => {
            console.error('Error fetching device options:', error);
            return of({ operations: [] });
          })
        );
      })
    ),
    { initialValue: { operations: [] } }
  );

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
    this.deviceIdInput.set('');
    this.deviceSearch = '';
    this.subProperty = '';
    this.dataSourceId = '';
    this.name = '';
    this.axisMode = 'merged';
    this.multiEntries = [
      {
        deviceId: '',
        deviceSearch: '',
        subProperty: '',
        options: [],
        suggestions: [],
      },
    ];
  }

  isFormValid(): boolean {
    if (!this.selectedComponentType) return false;

    if (!this.name) return false;

    if (this.selectedComponentType === 'graph') {
      return this.deviceIdInput().trim() !== '';
    }

    if (this.selectedComponentType === 'multiGraph') {
      // Require at least 2 valid device IDs
      const validCount = this.multiEntries.filter(
        e => e.deviceId.trim() !== ''
      ).length;
      return validCount >= 2;
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

  addEntry() {
    this.multiEntries.push({
      deviceId: '',
      deviceSearch: '',
      subProperty: '',
      options: [],
      suggestions: [],
    });
  }

  removeEntry(index: number) {
    if (this.multiEntries.length > 1) {
      this.multiEntries.splice(index, 1);
    }
  }

  onEntryDeviceIdChange(index: number, deviceId: string) {
    const trimmed = (deviceId || '').trim();
    if (trimmed === '') {
      this.multiEntries[index].options = [];
      this.multiEntries[index].subProperty = '';
      return;
    }
    this.deviceService.getDeviceOptions(trimmed).subscribe({
      next: res => {
        this.multiEntries[index].options = res.operations ?? [];
        // Clear subProperty if it's not in the allowed list anymore
        if (
          this.multiEntries[index].subProperty &&
          !this.multiEntries[index].options.includes(
            this.multiEntries[index].subProperty
          )
        ) {
          this.multiEntries[index].subProperty = '';
        }
      },
      error: () => {
        this.multiEntries[index].options = [];
      },
    });
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

  // Single graph autocomplete handlers
  onDeviceQuery(event: { query: string }) {
    const q = (event?.query || '').toLowerCase();
    const filtered = this.devices()
      .filter(
        d =>
          d.name.toLowerCase().includes(q) ||
          d.id.toLowerCase().includes(q) ||
          (d.address || '').toLowerCase().includes(q)
      )
      .slice(0, 20);
    this.deviceSuggestions.set(filtered);
  }

  onDeviceSelect(event: { value: DeviceResponseDto }) {
    const d = event?.value;
    if (d?.id) {
      this.deviceIdInput.set(d.id);
    }
  }

  onDeviceSearchChange(value: any) {
    // If user selected an option, it may be an object
    if (value && typeof value === 'object' && 'id' in value) {
      const selected = value as DeviceResponseDto;
      this.deviceIdInput.set(selected.id);
      return;
    }
    const str = (value as string) || '';
    const trimmed = str.trim();
    // Accept any non-empty id input
    if (trimmed !== '') {
      this.deviceIdInput.set(trimmed);
    }
  }

  // Multi graph autocomplete handlers
  onEntryDeviceQuery(index: number, event: { query: string }) {
    const q = (event?.query || '').toLowerCase();
    const filtered = this.devices()
      .filter(
        d =>
          d.name.toLowerCase().includes(q) ||
          d.id.toLowerCase().includes(q) ||
          (d.address || '').toLowerCase().includes(q)
      )
      .slice(0, 20);
    this.multiEntries[index].suggestions = filtered;
  }

  onEntryDeviceSelect(index: number, event: { value: DeviceResponseDto }) {
    const d = event?.value;
    if (d?.id) {
      this.multiEntries[index].deviceId = d.id;
      this.onEntryDeviceIdChange(index, d.id);
    }
  }

  onEntryDeviceSearchChange(index: number, value: any) {
    // If user selected an option, it may be an object
    if (value && typeof value === 'object' && 'id' in value) {
      const selected = value as DeviceResponseDto;
      this.multiEntries[index].deviceId = selected.id;
      this.onEntryDeviceIdChange(index, selected.id);
      return;
    }
    const str = (value as string) || '';
    const trimmed = str.trim();
    // Accept any non-empty id input
    if (trimmed !== '') {
      this.multiEntries[index].deviceId = trimmed;
      this.onEntryDeviceIdChange(index, trimmed);
    }
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
    } else if (this.selectedComponentType === 'multiGraph') {
      component.axisMode = this.axisMode;
      component.graphData = this.multiEntries
        .filter(e => e.deviceId.trim() !== '')
        .map(e => ({
          deviceId: e.deviceId.trim(),
          subProperty: e.subProperty.trim(),
        }));
    } else if (this.selectedComponentType === 'alertList') {
      component.dataSourceId = this.dataSourceId.trim();
    }

    this.componentAdded.emit(component);
    this.onCloseDialog();
  }
}
