import { Component, computed, inject, OnInit, signal } from '@angular/core';

import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { SortEvent } from 'primeng/api';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { TableModule } from 'primeng/table';
import { forkJoin, of } from 'rxjs';
import { catchError, map } from 'rxjs/operators';
import { DeviceResponseDto, DeviceStatusDto } from '../../dtos/device';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { DeviceService } from '../services/device-service';

@Component({
  selector: 'grn-device-overview',
  standalone: true,
  imports: [
    InputTextModule,
    FormsModule,
    TableModule,
    ButtonModule,
    NavBarComponent,
  ],
  templateUrl: './device-overview.component.html',
  styleUrl: './device-overview.component.scss',
})
export class DeviceOverviewComponent implements OnInit {
  devices = signal<DeviceResponseDto[]>([]);
  filteredDevices = computed(() =>
    this.devices().filter(device =>
      device.name.toLowerCase().includes(this.searchTerm().toLowerCase())
    )
  );
  loading = signal(true);
  searchTerm = signal('');

  private deviceService = inject(DeviceService);
  private router = inject(Router);

  ngOnInit() {
    this.loadDevices();
  }

  loadDevices() {
    this.loading.set(true);
    this.deviceService.getAllDevices().subscribe({
      next: devices => {
        // Load status for each device
        const statusRequests = devices.map(device =>
          this.deviceService.getDeviceStatus(device.id).pipe(
            map(statusResponse => ({ device, status: statusResponse.status })),
            catchError(() => of({ device, status: undefined }))
          )
        );

        if (statusRequests.length === 0) {
          this.devices.set([]);
          this.loading.set(false);
          return;
        }

        forkJoin(statusRequests).subscribe({
          next: results => {
            const devicesWithStatus = results.map(result => ({
              ...result.device,
              status: result.status,
            }));
            devicesWithStatus.sort((a, b) => {
              if (a.status === undefined) return 1;
              if (b.status === undefined) return -1;
              return a.status.localeCompare(b.status);
            });
            this.devices.set(devicesWithStatus);
            this.loading.set(false);
          },
          error: error => {
            console.error('Error loading device statuses:', error);
            this.devices.set(devices);
            this.loading.set(false);
          },
        });
      },
      error: error => {
        console.error('Error loading devices:', error);
        this.loading.set(false);
      },
    });
  }

  getStatusColor(status: DeviceStatusDto): string {
    switch (status) {
      case DeviceStatusDto.Online:
        return 'green';
      case DeviceStatusDto.Panic:
        return 'red';
      default:
        return 'gray';
    }
  }

  getStatusText(status: DeviceStatusDto): string {
    switch (status) {
      case DeviceStatusDto.Online:
        return 'Online';
      case DeviceStatusDto.Panic:
        return 'Panic';
      default:
        return 'Offline';
    }
  }

  addNewDevice() {
    this.router.navigate(['/smart_devices/add']);
  }

  navigateToDeviceDetail(device: DeviceResponseDto) {
    this.router.navigate(['/smart_devices', device.id]);
  }

  customSort(event: SortEvent) {
    event.data!.sort((data1, data2) => {
      let value1: unknown;
      let value2: unknown;

      if (event.field === 'status') {
        value1 = data1.status || 'Unknown';
        value2 = data2.status || 'Unknown';
      } else {
        value1 = data1[event.field!];
        value2 = data2[event.field!];
      }

      const result =
        (value1 as number) < (value2 as number)
          ? -1
          : (value1 as number) > (value2 as number)
            ? 1
            : 0;
      return event.order! * result;
    });
  }
}
