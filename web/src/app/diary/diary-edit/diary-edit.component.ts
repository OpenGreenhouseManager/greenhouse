import { Location } from '@angular/common';
import { Component, computed, inject, linkedSignal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { form, FormField } from '@angular/forms/signals';
import { ActivatedRoute, Router } from '@angular/router';
import { ButtonDirective } from 'primeng/button';
import { DatePicker } from 'primeng/datepicker';
import { InputTextModule } from 'primeng/inputtext';
import { TextareaModule } from 'primeng/textarea';
import { CardComponent } from '../../card/card.component';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { Diary } from '../models/diary';
import { DiaryService } from '../services/diary-service';

@Component({
  selector: 'grn-diary-edit',
  imports: [
    NavBarComponent,
    CardComponent,
    FormField,
    InputTextModule,
    ButtonDirective,
    TextareaModule,
    DatePicker,
  ],
  templateUrl: './diary-edit.component.html',
  styleUrl: './diary-edit.component.scss',
})
export class DiaryEditComponent {
  private router = inject(Router);
  private route = inject(ActivatedRoute);
  private diaryService = inject(DiaryService);
  private location = inject(Location);

  private id = this.route.snapshot.paramMap.get('id');
  edit = computed(() => this.id !== null);
  private loadedDiary = toSignal(this.diaryService.getDiary(this.id), {
    initialValue: new Diary(new Date(), '', '', new Date(), new Date()),
  });
  private diary = linkedSignal(() => this.loadedDiary());
  diaryForm = form(this.diary);

  goBack() {
    this.location.back();
  }

  saveEntry() {
    let observable;
    const diary = this.diary();
    if (!diary) {
      return;
    }
    if (this.edit()) {
      observable = this.diaryService.updateDiaryEntry(this.id ?? '', diary);
    } else {
      observable = this.diaryService.addDiaryEntry(diary);
    }
    observable.subscribe(() => this.router.navigate(['/diary']));
  }
}
