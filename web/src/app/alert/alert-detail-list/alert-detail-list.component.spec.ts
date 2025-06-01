import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';
import { AlertDetailListComponent } from './alert-detail-list.component';

describe('AlertDetailListComponent', () => {
  let component: AlertDetailListComponent;
  let fixture: ComponentFixture<AlertDetailListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertDetailListComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient()],
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
