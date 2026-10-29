import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideRouter } from '@angular/router';
import { AlertDetailListComponent } from './alert-detail-list.component';

describe('AlertDetailListComponent', () => {
  let component: AlertDetailListComponent;
  let fixture: ComponentFixture<AlertDetailListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertDetailListComponent],
      providers: [provideRouter([]), provideHttpClient()],
    }).compileComponents();

    fixture = TestBed.createComponent(AlertDetailListComponent);
    component = fixture.componentInstance;
    fixture.componentRef.setInput('alerts', []);
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
