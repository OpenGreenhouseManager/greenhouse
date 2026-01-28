import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { RouterModule } from '@angular/router';
import { AlertOverviewComponent } from './alert-overview.component';

describe('AlertOverviewComponent', () => {
  let component: AlertOverviewComponent;
  let fixture: ComponentFixture<AlertOverviewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertOverviewComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();

    fixture = TestBed.createComponent(AlertOverviewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
