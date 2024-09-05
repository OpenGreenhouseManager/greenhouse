import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DiaryDetailComponent } from './diary-detail.component';
import { provideHttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';

describe('DiaryDetailComponent', () => {
  let component: DiaryDetailComponent;
  let fixture: ComponentFixture<DiaryDetailComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DiaryDetailComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient()],
    }).compileComponents();

    fixture = TestBed.createComponent(DiaryDetailComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
