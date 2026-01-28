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
import { catchError, map, of, switchMap } from 'rxjs';
import { DeviceService } from '../device';
import type { DeviceResponseDto } from '../dtos/device';

@Component({
  selector: 'grn-single-data-form',
  standalone: true,
  imports: [AutoComplete, FormsModule],
  host: {
    class: 'flex gap-2',
    '[class.flex-row]': 'compact()',
    '[class.flex-col]': '!compact()',
  },
  template: `
    <div class="flex items-center gap-4" [class.mb-4]="!compact()">
      @if (!compact()) {
        <label for="deviceId" class="font-semibold w-24">Device</label>
      }
      <p-autocomplete
        id="deviceId"
        class="w-full"
        [suggestions]="deviceSuggestions()"
        (completeMethod)="onDeviceQuery($event)"
        [(ngModel)]="deviceSearch"
        [optionLabel]="'name'"
        placeholder="Select Device"
        [dropdown]="true"
        [appendTo]="'body'"
        (onSelect)="onDeviceSelect($event)"
        (ngModelChange)="onDeviceSearchChange($event)" />
    </div>

    <div class="flex items-center gap-4" [class.mb-4]="!compact()">
      @if (!compact()) {
        <label for="subProperty" class="font-semibold w-24">Sub Property</label>
      }

      <p-autocomplete
        id="subProperty"
        class="w-full"
        [suggestions]="subPropertySuggestions()"
        (completeMethod)="onSubPropertyQuery($event)"
        [(ngModel)]="subProperty"
        [optionLabel]="'display'"
        [optionValue]="'value'"
        placeholder="Select Sub Property"
        [dropdown]="true"
        [appendTo]="'body'"
        [group]="true"
        [optionGroupLabel]="'prefix'"
        [optionGroupChildren]="'operations'"
        [disabled]="
          !validDeviceId() || subPropertyOptions().operations.length === 0
        " />
    </div>
  `,
})
export class SingleDataFormComponent {
  deviceService = inject(DeviceService);

  devices = input.required<DeviceResponseDto[]>();
  compact = input<boolean>(false);

  selectedDevice = output<{
    deviceId: string;
    subProperty: string;
  }>();

  deviceSuggestions = signal<DeviceResponseDto[]>([]);
  deviceSearch = signal<string>('');
  deviceId = signal<string>('');
  subProperty = signal<string>('');
  subPropertySuggestions = signal<
    { prefix: string; operations: { display: string; value: string }[] }[]
  >([]);

  subPropertyOptions = toSignal(
    toObservable(this.deviceId).pipe(
      switchMap(deviceId => {
        if (!deviceId) return of({ operations: [] });
        return this.deviceService.getDeviceOptions(deviceId).pipe(
          map(options => ({ operations: options.operations.sort() })),
          catchError(() => {
            return of({ operations: [] });
          })
        );
      })
    ),
    { initialValue: { operations: [] } }
  );

  validDeviceId = computed(() => {
    const input = this.deviceId().trim();
    return input !== '';
  });

  constructor() {
    effect(() => {
      if (this.validDeviceId() && this.subProperty()?.trim() !== '') {
        this.selectedDevice.emit({
          deviceId: this.deviceId(),
          subProperty: this.subProperty(),
        });
      }
    });
  }

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
      this.deviceId.set(d.id);
    }
  }

  onDeviceSearchChange(value: unknown) {
    // If user selected an option, it may be an object
    if (value && typeof value === 'object' && 'id' in value) {
      const selected = value as DeviceResponseDto;
      this.deviceId.set(selected.id);
      return;
    }
    const str = (value as string) || '';
    const trimmed = str.trim();
    // Accept any non-empty id input
    if (trimmed !== '') {
      this.deviceId.set(trimmed);
    }
  }

  onSubPropertyQuery(event: { query: string }) {
    const q = (event?.query || '').toLowerCase();
    const filtered = this.subPropertyOptions().operations.filter(op =>
      op.toLowerCase().includes(q)
    );
    this.subPropertySuggestions.set(this.splitIntoGroups(filtered));
  }

  splitIntoGroups(
    operations: string[]
  ): { prefix: string; operations: { display: string; value: string }[] }[] {
    const groups = new Set<string>();
    for (const op of operations) {
      const prefix = op.split('_')[0];
      if (!prefix) {
        continue;
      }
      groups.add(prefix);
    }
    return Array.from(groups).map(prefix => ({
      prefix,
      operations: operations
        .filter(op => op.split('_')[0] === prefix)
        .map(op => ({ display: op.replace(`${prefix}_`, ''), value: op }))
        .sort((a, b) => a.display.localeCompare(b.display)),
    }));
  }
}
