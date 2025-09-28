import { Location } from '@angular/common';
import { Component, computed, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { ButtonModule } from 'primeng/button';
import { CheckboxModule } from 'primeng/checkbox';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputTextModule } from 'primeng/inputtext';
import { TextareaModule } from 'primeng/textarea';
import { map, of } from 'rxjs';
import { CardComponent } from '../../card/card.component';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { Device } from '../models/device';
import { DeviceService } from '../services/device-service';

@Component({
  selector: 'grn-device-edit',
  standalone: true,
  imports: [
    NavBarComponent,
    CardComponent,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    TextareaModule,
    CheckboxModule,
  ],
  templateUrl: './device-edit.component.html',
  styleUrl: './device-edit.component.scss',
})
export class DeviceEditComponent {
  private router = inject(Router);
  private route = inject(ActivatedRoute);
  private deviceService = inject(DeviceService);
  private location = inject(Location);

  private id = this.route.snapshot.paramMap.get('id');
  edit = computed(() => this.id !== null);

  // Create a reactive device object for editing
  device = toSignal(
    this.edit()
      ? this.deviceService
          .getDeviceById(this.id ?? '')
          .pipe(
            map(
              deviceDto =>
                new Device(
                  deviceDto.name,
                  deviceDto.description,
                  deviceDto.address,
                  deviceDto.canscript,
                  deviceDto.scraping
                )
            )
          )
      : of(new Device()),
    {
      initialValue: new Device(),
    }
  );

  goBack() {
    this.location.back();
  }

  saveDevice() {
    const deviceData = this.device();
    if (!deviceData) return;

    let observable;
    if (this.edit()) {
      observable = this.deviceService.updateDevice(this.id ?? '', {
        name: deviceData.name,
        description: deviceData.description,
        address: deviceData.address,
        can_script: deviceData.can_script,
      });
    } else {
      observable = this.deviceService.addDevice({
        name: deviceData.name,
        description: deviceData.description,
        address: deviceData.address,
        can_script: deviceData.can_script,
        scraping: deviceData.scraping,
      });
    }
    observable.subscribe(response =>
      this.router.navigate(['/smart_devices', this.id])
    );
  }
}
