import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DiaryEditComponent } from './diary-edit.component';
import { provideHttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';

describe('DiaryEditComponent', () => {
  let component: DiaryEditComponent;
  let fixture: ComponentFixture<DiaryEditComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DiaryEditComponent, RouterModule.forRoot([])],
      providers: [provideHttpClient()],
    }).compileComponents();

    fixture = TestBed.createComponent(DiaryEditComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
