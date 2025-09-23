import { Component, computed, OnInit, signal } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { CardComponent } from '../../card/card.component';
import { DeviceService } from '../services/device-service';
import {
  DeviceResponseDto,
  ConfigResponseDto,
  DeviceStatusDto,
  Type,
  Mode,
} from '../../dtos/device';
import { catchError, forkJoin, of } from 'rxjs';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { MessageModule } from 'primeng/message';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { NgxJsonViewerModule } from 'ngx-json-viewer';
import { ButtonModule } from 'primeng/button';
import { GraphComponent } from '../../shared/graph/graph.component';
import { AlertListComponent } from '../../shared/alert/alert-list.component';

@Component({
  selector: 'app-device-detail',
  standalone: true,
  imports: [
    CommonModule,
    CardComponent,
    ProgressSpinnerModule,
    MessageModule,
    NavBarComponent,
    NgxJsonViewerModule,
    ButtonModule,
    GraphComponent,
    AlertListComponent,
  ],
  templateUrl: './device-detail.component.html',
  styleUrl: './device-detail.component.scss',
})
export class DeviceDetailComponent implements OnInit {
  device = signal<DeviceResponseDto | null>(null);
  deviceConfig = signal<ConfigResponseDto | null>(null);
  loading = signal(true);
  error = signal<string | null>(null);
  loadingActivation = signal(false);

  public hasAlert = computed(() => {
    return this.dataSourceId() !== null;
  });

  public config = computed(() => {
    if (!this.device()) {
      return null;
    }
    return {
      device_id: this.device()!.id,
    };
  });

  public hasGraph = computed(() => {
    return (
      this.device()?.scraping &&
      this.deviceConfig()?.output_type === Type.Number &&
      (this.deviceConfig()?.mode === Mode.Output ||
        this.deviceConfig()?.mode === Mode.InputOutput)
    );
  });

  public dataSourceId = signal<string | null>(null);

  constructor(
    private deviceService: DeviceService,
    private router: Router,
    private route: ActivatedRoute
  ) {}

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
}
