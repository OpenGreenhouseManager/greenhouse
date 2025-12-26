import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';

import { toObservable, toSignal } from '@angular/core/rxjs-interop';
import { NgxJsonViewerModule } from 'ngx-json-viewer';
import { FormsModule } from '@angular/forms';
import { ButtonModule } from 'primeng/button';
import { MessageModule } from 'primeng/message';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { Dialog } from 'primeng/dialog';
import { TextareaModule } from 'primeng/textarea';
import { catchError, combineLatest, forkJoin, of, switchMap } from 'rxjs';
import { CardComponent } from '../../card/card.component';
import {
  ConfigResponseDto,
  DeviceResponseDto,
  DeviceStatusDto,
  Mode,
  Type,
} from '../../dtos/device';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { AlertListComponent } from '../../shared/alert/alert-list.component';
import {
  GraphComponent,
  GraphConfig,
} from '../../shared/graph/graph.component';
import { DeviceService } from '../services/device-service';

@Component({
  selector: 'app-device-detail',
  standalone: true,
  imports: [
    CardComponent,
    ProgressSpinnerModule,
    MessageModule,
    NavBarComponent,
    FormsModule,
    NgxJsonViewerModule,
    ButtonModule,
    Dialog,
    TextareaModule,
    GraphComponent,
    AlertListComponent,
  ],
  templateUrl: './device-detail.component.html',
  styleUrl: './device-detail.component.scss',
})
export class DeviceDetailComponent implements OnInit {
  private deviceService = inject(DeviceService);
  private router = inject(Router);
  private route = inject(ActivatedRoute);

  device = signal<DeviceResponseDto | null>(null);
  deviceConfig = signal<ConfigResponseDto | null>(null);
  loading = signal(true);
  error = signal<string | null>(null);
  loadingActivation = signal(false);

  // Edit Additional Config dialog state
  editAdditionalConfigVisible = signal(false);
  additionalConfigDraft = signal<string>('');
  saveConfigLoading = signal(false);
  saveConfigError = signal<string | null>(null);
  originalAdditionalConfigKeys = signal<string[]>([]);

  public hasAlert = computed(() => {
    return this.dataSourceId() !== null;
  });

  public configList = toSignal(
    combineLatest([
      toObservable(this.device),
      toObservable(this.deviceConfig),
    ]).pipe(
      switchMap(([device, deviceConfig]) => {
        if (!device) return of({ operations: [] });
        if (!deviceConfig) return of({ operations: [] });

        if (deviceConfig.output_type === Type.Number) {
          return of({ operations: [] });
        }
        return this.deviceService.getDeviceOptions(device.id);
      })
    )
  );

  public configs = computed(() => {
    const configList = this.configList();
    const device = this.device();

    if (!configList || !device || !configList.operations) {
      return [];
    }

    return configList.operations.map((option: string, index: number) => {
      return {
        trackingId: index,
        config: {
          graph_data: [
            {
              device_id: device.id,
              sub_property: option ?? undefined,
            },
          ],
          axis_mode: 'merged',
        } satisfies GraphConfig,
      };
    });
  });

  public hasGraph = computed(() => {
    return (
      this.device()?.scraping &&
      (this.deviceConfig()?.mode === Mode.Output ||
        this.deviceConfig()?.mode === Mode.InputOutput)
    );
  });

  public dataSourceId = signal<string | null>(null);

  ngOnInit(): void {
    const deviceId = this.route.snapshot.paramMap.get('id');
    if (deviceId) {
      this.loadDeviceData(deviceId);
    } else {
      this.error.set('Device ID not found');
      this.loading.set(false);
    }
  }

  private loadDeviceData(deviceId: string): void {
    this.loading.set(true);
    this.error.set(null);

    forkJoin({
      device: this.deviceService.getDeviceById(deviceId).pipe(
        catchError(error => {
          console.error('Error loading device:', error);
          return of(null);
        })
      ),
      config: this.deviceService.getDeviceConfig(deviceId).pipe(
        catchError(error => {
          console.error('Error loading device config:', error);
          return of(null);
        })
      ),
      status: this.deviceService.getDeviceStatus(deviceId).pipe(
        catchError(error => {
          console.error('Error loading device status:', error);
          return of(null);
        })
      ),
    }).subscribe({
      next: result => {
        this.device.set(result.device);
        this.deviceConfig.set(result.config);
        if (this.device()) {
          this.device()!.status = result.status?.status;
        }
        this.dataSourceId.set(result.status?.datasource_id || null);
        this.loading.set(false);

        if (!result.device && !result.config) {
          this.error.set('Failed to load device data');
        }
      },
      error: error => {
        console.error('Error loading device data:', error);
        this.error.set('An error occurred while loading device data');
        this.loading.set(false);
      },
    });
  }

  editDevice(): void {
    if (this.device()) {
      this.router.navigate(['/smart_devices', this.device()!.id, 'edit']);
    }
  }

  goBack() {
    this.router.navigate(['/smart_devices']);
  }

  getStatusColor(status?: DeviceStatusDto): string {
    if (!status) {
      return 'gray';
    }
    switch (status) {
      case DeviceStatusDto.Online:
        return 'green';
      case DeviceStatusDto.Panic:
        return 'red';
      default:
        return 'gray';
    }
  }

  getStatusText(status?: DeviceStatusDto): string {
    if (!status) {
      return 'Offline';
    }
    switch (status) {
      case DeviceStatusDto.Online:
        return 'Online';
      case DeviceStatusDto.Panic:
        return 'Panic';
      default:
        return 'Offline';
    }
  }

  registerDevice(): void {
    if (this.device()) {
      this.deviceService.registerDevice(this.device()!.id);
      this.loadingActivation.set(true);
      setTimeout(() => {
        this.deviceService.getDeviceConfig(this.device()!.id).subscribe({
          next: response => {
            this.deviceConfig()!.scripting_api = response.scripting_api;
            this.loadingActivation.set(true);
          },
          error: error => {
            console.error(error);
            this.loadingActivation.set(false);
          },
        });
      }, 1000);
    }
  }

  openEditAdditionalConfig(): void {
    const current = this.deviceConfig()?.additional_config ?? {};
    try {
      this.additionalConfigDraft.set(JSON.stringify(current, null, 2));
    } catch {
      this.additionalConfigDraft.set('');
    }
    // capture original top-level keys if object
    if (current && typeof current === 'object' && !Array.isArray(current)) {
      this.originalAdditionalConfigKeys.set(
        Object.keys(current as Record<string, unknown>)
      );
    } else {
      this.originalAdditionalConfigKeys.set([]);
    }
    this.saveConfigError.set(null);
    this.editAdditionalConfigVisible.set(true);
  }

  onCancelEditAdditional(): void {
    this.editAdditionalConfigVisible.set(false);
    this.saveConfigLoading.set(false);
    this.saveConfigError.set(null);
  }

  saveAdditionalConfig(): void {
    if (!this.device()) {
      this.saveConfigError.set('Device not loaded');
      return;
    }
    let parsed: unknown;
    try {
      parsed =
        this.additionalConfigDraft() === ''
          ? {}
          : JSON.parse(this.additionalConfigDraft());
    } catch (e) {
      this.saveConfigError.set('Invalid JSON. Please fix and try again.');
      return;
    }

    // Enforce same top-level keys if we had an object with known keys
    const requiredKeys = this.originalAdditionalConfigKeys();
    if (
      requiredKeys.length > 0 &&
      parsed &&
      typeof parsed === 'object' &&
      !Array.isArray(parsed)
    ) {
      const newKeys = Object.keys(parsed as Record<string, unknown>);
      const missing = requiredKeys.filter(k => !newKeys.includes(k));
      const extra = newKeys.filter(k => !requiredKeys.includes(k));
      if (missing.length > 0 || extra.length > 0) {
        const parts: string[] = [];
        if (missing.length > 0) parts.push(`missing: ${missing.join(', ')}`);
        if (extra.length > 0) parts.push(`unexpected: ${extra.join(', ')}`);
        this.saveConfigError.set(
          `Keys must match original set (${requiredKeys.join(', ')}); ${parts.join(' | ')}`
        );
        return;
      }
    }

    this.saveConfigLoading.set(true);
    this.saveConfigError.set(null);
    this.deviceService
      .updateDeviceAdditionalConfig(this.device()!.id, parsed)
      .subscribe({
        next: () => {
          // update local state and close dialog
          const cfg = this.deviceConfig();
          if (cfg) {
            (cfg as ConfigResponseDto).additional_config =
              parsed as unknown as Record<string, unknown>;
            this.deviceConfig.set({ ...cfg });
          }
          this.saveConfigLoading.set(false);
          this.editAdditionalConfigVisible.set(false);
        },
        error: error => {
          console.error('Failed to update additional config', error);
          this.saveConfigError.set('Failed to save configuration');
          this.saveConfigLoading.set(false);
        },
      });
  }

  formatAdditionalConfig(): void {
    try {
      const parsed =
        this.additionalConfigDraft() === ''
          ? {}
          : JSON.parse(this.additionalConfigDraft());
      this.additionalConfigDraft.set(JSON.stringify(parsed, null, 2));
      this.saveConfigError.set(null);
    } catch {
      this.saveConfigError.set(
        'Cannot format: current content is not valid JSON.'
      );
    }
  }
}
