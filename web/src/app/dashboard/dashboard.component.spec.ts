import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import { UserPreferencesService } from '../services/user-preferences.service';
import { DashboardComponent } from './dashboard.component';

describe('DashboardComponent', () => {
  let component: DashboardComponent;
  let fixture: ComponentFixture<DashboardComponent>;

  beforeEach(async () => {
    const preferences = {
      dashboard_preferences: JSON.stringify([{ name: 'Tab 1', items: [] }]),
      alert_preferences: '{}',
    };
    await TestBed.configureTestingModule({
      imports: [DashboardComponent],
      providers: [
        provideRouter([]),
        provideHttpClient(),
        provideHttpClientTesting(),
        {
          provide: UserPreferencesService,
          useValue: {
            getUserPreferences: () => of(preferences),
            updateUserPreferences: () => of(preferences),
          },
        },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(DashboardComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('renders a newly added component in the active tab', async () => {
    const items = () =>
      fixture.nativeElement.querySelectorAll('gridster-item').length;
    expect(items()).toBe(0);

    component.onComponentAdded({
      type: 'alertList',
      dataSourceId: 'source-1',
      name: 'Alerts',
    });
    await fixture.whenStable();

    expect(items()).toBe(1);
  });
});
