import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideRouter } from '@angular/router';
import { DeviceOverviewComponent } from './device-overview.component';

describe('DeviceOverviewComponent', () => {
  let component: DeviceOverviewComponent;
  let fixture: ComponentFixture<DeviceOverviewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DeviceOverviewComponent],
      providers: [provideHttpClient(), provideRouter([])],
    }).compileComponents();

    fixture = TestBed.createComponent(DeviceOverviewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
