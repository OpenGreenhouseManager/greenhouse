import type { ComponentFixture } from '@angular/core/testing';
import { TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { RouterModule } from '@angular/router';
import { DiaryOverviewComponent } from './diary-overview.component';

describe('DiaryOverviewComponent', () => {
  let component: DiaryOverviewComponent;
  let fixture: ComponentFixture<DiaryOverviewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DiaryOverviewComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    }).compileComponents();

    fixture = TestBed.createComponent(DiaryOverviewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
