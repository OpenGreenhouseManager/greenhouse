import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DiaryOverviewComponent } from './diary-overview.component';
import { provideHttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';

describe('DiaryOverviewComponent', () => {
  let component: DiaryOverviewComponent;
  let fixture: ComponentFixture<DiaryOverviewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DiaryOverviewComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient()],
    }).compileComponents();

    fixture = TestBed.createComponent(DiaryOverviewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
