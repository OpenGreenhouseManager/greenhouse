import { CommonModule } from '@angular/common';
import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import {
  addDays,
  endOfDay,
  isToday,
  isYesterday,
  startOfDay,
  subDays,
} from 'date-fns';
import { ButtonModule } from 'primeng/button';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { InputTextModule } from 'primeng/inputtext';
import { CardComponent } from '../../card/card.component';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { Diary } from '../models/diary';
import { DiaryService } from '../services/diary-service';

@Component({
  selector: 'grn-diary-overview',
  standalone: true,
  imports: [
    NavBarComponent,
    CommonModule,
    CardComponent,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    InputGroupAddonModule,
  ],
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

  diaryClicked(diaryId: string | undefined) {
    this.router.navigate(['/diary', diaryId]);
  }
}
