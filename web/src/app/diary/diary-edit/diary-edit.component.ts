import { Component, computed, OnInit, signal } from '@angular/core';
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
import { InputTextareaModule } from 'primeng/inputtextarea';

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
    InputTextareaModule,
  ],
  templateUrl: './diary-edit.component.html',
  styleUrl: './diary-edit.component.scss',
})
export class DiaryEditComponent implements OnInit {
  private id = this.route.snapshot.paramMap.get('id');
  edit = computed(() => this.id !== null);
  diary = signal(new Diary(new Date(), '', '', new Date(), new Date()));

  constructor(
    private router: Router,
    private route: ActivatedRoute,
    private diaryService: DiaryService
  ) {}

  ngOnInit() {
    if (this.edit()) {
      this.diary.set(this.diaryService.getDiary(this.id as string));
    }
  }

  saveEntry() {
    if (this.edit()) {
      this.diaryService.updateDiaryEntry(this.id ?? '', this.diary());
    } else {
      this.diaryService.addDiaryEntry(this.diary());
    }
    this.router.navigate(['/diary']);
  }
}
