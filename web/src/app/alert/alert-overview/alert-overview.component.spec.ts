import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AlertOverviewComponent } from './alert-overview.component';
import { provideHttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';

describe('AlertOverviewComponent', () => {
  let component: AlertOverviewComponent;
  let fixture: ComponentFixture<AlertOverviewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertOverviewComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient()],
    }).compileComponents();

    fixture = TestBed.createComponent(AlertOverviewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
