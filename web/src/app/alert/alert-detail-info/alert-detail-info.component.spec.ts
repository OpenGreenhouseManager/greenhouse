import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';

import { AlertDetailInfoComponent } from './alert-detail-info.component';
import { provideHttpClient } from '@angular/common/http';
import { provideRouter } from '@angular/router';
import { provideEchartsCore } from 'ngx-echarts';
import { echarts } from '../../shared/graph/echarts';

describe('AlertDetailInfoComponent', () => {
  let component: AlertDetailInfoComponent;
  let fixture: ComponentFixture<AlertDetailInfoComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertDetailInfoComponent],
      providers: [
        provideRouter([]),
        provideHttpClient(),
        provideEchartsCore({ echarts }),
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(AlertDetailInfoComponent);
    component = fixture.componentInstance;
    fixture.componentRef.setInput('alerts', []);
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
