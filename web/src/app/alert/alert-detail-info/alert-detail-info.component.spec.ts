import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AlertDetailInfoComponent } from './alert-detail-info.component';
import { provideHttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';

describe('AlertDetailInfoComponent', () => {
  let component: AlertDetailInfoComponent;
  let fixture: ComponentFixture<AlertDetailInfoComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertDetailInfoComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient()],
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
