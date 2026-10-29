import { KeyValuePipe } from '@angular/common';
import type { OnInit } from '@angular/core';
import { Component, computed, inject, signal } from '@angular/core';
import { Router } from '@angular/router';
import {
  addDays,
  endOfDay,
  isToday,
  isYesterday,
  startOfDay,
  subDays,
} from 'date-fns';
import { ButtonDirective } from 'primeng/button';
import { CardComponent } from '../../card/card.component';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import type { Diary } from '../models/diary';
import { DiaryService } from '../services/diary-service';

@Component({
  selector: 'grn-diary-overview',
  imports: [NavBarComponent, CardComponent, ButtonDirective, KeyValuePipe],
  templateUrl: './diary-overview.component.html',
  styleUrl: './diary-overview.component.scss',
})
export class DiaryOverviewComponent implements OnInit {
  diaries = signal(new Map<string, Diary>());
  selectedDate = signal(new Date());
  title = computed(() => {
    if (isToday(this.selectedDate())) {
      return 'Today';
    }
    if (isYesterday(this.selectedDate())) {
      return 'Yesterday';
    }
    return this.selectedDate().toLocaleDateString();
  });

  private router = inject(Router);
  private diaryService = inject(DiaryService);

  ngOnInit() {
    this.updateDiaries();
  }

  addEntry() {
    this.router.navigate(['/diary/add']);
  }

  getTime(date: Date | undefined) {
    if (!date) {
      return '';
    }
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  subDate() {
    this.selectedDate.set(subDays(this.selectedDate(), 1));
    this.updateDiaries();
  }

  addDate() {
    if (isToday(this.selectedDate())) {
      return;
    }
    this.selectedDate.set(addDays(this.selectedDate(), 1));
    this.updateDiaries();
  }

  updateDiaries() {
    this.diaryService
      .getDiaries(
        startOfDay(this.selectedDate()),
        endOfDay(this.selectedDate())
      )
      .subscribe(value => {
        this.diaries.set(value);
      });
  }

  diaryClicked(diaryId: string) {
    this.router.navigate(['/diary', diaryId]);
  }
}
