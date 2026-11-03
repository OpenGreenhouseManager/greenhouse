import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import {
  ActivatedRoute,
  convertToParamMap,
  provideRouter,
} from '@angular/router';
import { of } from 'rxjs';
import { DeviceService } from '../services/device-service';
import { DeviceEditComponent } from './device-edit.component';

describe('DeviceEditComponent', () => {
  let component: DeviceEditComponent;
  let fixture: ComponentFixture<DeviceEditComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DeviceEditComponent],
      providers: [provideHttpClient(), provideRouter([])],
    }).compileComponents();

    fixture = TestBed.createComponent(DeviceEditComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

describe('DeviceEditComponent delete', () => {
  let component: DeviceEditComponent;
  const deleteDevice = vi.fn(() => of(undefined));

  beforeEach(async () => {
    deleteDevice.mockClear();
    await TestBed.configureTestingModule({
      imports: [DeviceEditComponent],
      providers: [
        provideRouter([{ path: '**', children: [] }]),
        {
          provide: ActivatedRoute,
          useValue: { snapshot: { paramMap: convertToParamMap({ id: '42' }) } },
        },
        {
          provide: DeviceService,
          useValue: {
            getDeviceById: () =>
              of({
                id: '42',
                name: 'Greenhouse Sensor',
                description: '',
                address: 'http://x',
                canscript: false,
                scraping: false,
              }),
            deleteDevice,
          },
        },
      ],
    }).compileComponents();

    const fixture = TestBed.createComponent(DeviceEditComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('only sends the request once the exact device name is typed', () => {
    component.openDeleteDialog();

    component.deleteConfirmation.set('greenhouse sensor');
    component.deleteDevice();
    expect(deleteDevice).not.toHaveBeenCalled();

    component.deleteConfirmation.set('Greenhouse Sensor');
    expect(component.deleteConfirmed()).toBe(true);
    component.deleteDevice();
    expect(deleteDevice).toHaveBeenCalledWith('42');
  });
});
