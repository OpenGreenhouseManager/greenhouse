import { Component, computed } from '@angular/core';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { CommonModule } from '@angular/common';
import { CardComponent } from '../../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { ActivatedRoute, Router } from '@angular/router';
import { DiaryService } from '../services/diary-service';
import { Diary } from '../models/diary';
import { TextareaModule } from 'primeng/textarea';
import { toSignal } from '@angular/core/rxjs-interop';
import { DatePicker } from 'primeng/datepicker';

@Component({
  selector: 'grn-diary-edit',
  standalone: true,
  imports: [
    NavBarComponent,
    CommonModule,
    CardComponent,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    TextareaModule,
    DatePicker,
  ],
  templateUrl: './diary-edit.component.html',
  styleUrl: './diary-edit.component.scss',
})
export class DiaryEditComponent {
  private id = this.route.snapshot.paramMap.get('id');
  edit = computed(() => this.id !== null);
  diary = toSignal(this.diaryService.getDiary(this.id), {
    initialValue: new Diary(new Date(), '', '', new Date(), new Date()),
    rejectErrors: true,
  });

  constructor(
    private router: Router,
    private route: ActivatedRoute,
    private diaryService: DiaryService
  ) {}

  saveEntry() {
    let observable;
    if (this.edit()) {
      observable = this.diaryService.updateDiaryEntry(
        this.id ?? '',
        this.diary()
      );
    } else {
      observable = this.diaryService.addDiaryEntry(this.diary());
    }
    observable.subscribe(() => this.router.navigate(['/diary']));
  }
}
