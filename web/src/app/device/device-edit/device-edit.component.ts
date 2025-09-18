import { Component, computed } from '@angular/core';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { CommonModule, Location } from '@angular/common';
import { CardComponent } from '../../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { ActivatedRoute, Router } from '@angular/router';
import { DeviceService } from '../services/device-service';
import { Device } from '../models/device';
import { TextareaModule } from 'primeng/textarea';
import { toSignal } from '@angular/core/rxjs-interop';
import { CheckboxModule } from 'primeng/checkbox';
import { map, of } from 'rxjs';

@Component({
  selector: 'grn-device-edit',
  standalone: true,
  imports: [
    NavBarComponent,
    CommonModule,
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
                  deviceDto.scraping,
                )
            )
          )
      : of(new Device()),
    {
      initialValue: new Device(),
      rejectErrors: true,
    }
  );

  constructor(
    private router: Router,
    private route: ActivatedRoute,
    private deviceService: DeviceService,
    private location: Location
  ) {}

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
      this.router.navigate(['/smart_devices', response.id])
    );
  }
}
