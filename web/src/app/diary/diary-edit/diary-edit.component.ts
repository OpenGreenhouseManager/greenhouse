import { Location } from '@angular/common';
import { Component, computed, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { ButtonModule } from 'primeng/button';
import { DatePicker } from 'primeng/datepicker';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputTextModule } from 'primeng/inputtext';
import { TextareaModule } from 'primeng/textarea';
import { CardComponent } from '../../card/card.component';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { Diary } from '../models/diary';
import { DiaryService } from '../services/diary-service';

@Component({
  selector: 'grn-diary-edit',
  standalone: true,
  imports: [
    NavBarComponent,
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
  private router = inject(Router);
  private route = inject(ActivatedRoute);
  private diaryService = inject(DiaryService);
  private location = inject(Location);

  private id = this.route.snapshot.paramMap.get('id');
  edit = computed(() => this.id !== null);
  diary = toSignal(this.diaryService.getDiary(this.id), {
    initialValue: new Diary(new Date(), '', '', new Date(), new Date()),
  });

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
