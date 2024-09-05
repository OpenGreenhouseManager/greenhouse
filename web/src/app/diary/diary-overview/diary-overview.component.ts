import { Component, computed, OnInit, signal } from '@angular/core';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { CommonModule } from '@angular/common';
import { CardComponent } from '../../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { Router } from '@angular/router';
import { DiaryService } from '../services/diary-service';
import {
  startOfDay,
  endOfDay,
  isToday,
  isYesterday,
  subDays,
  addDays,
} from 'date-fns';
import { Diary } from '../models/diary';

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
  constructor(
    private router: Router,
    private diaryService: DiaryService
  ) {}

  ngOnInit() {
    this.diaryService
      .getDiaries(
        startOfDay(this.selectedDate()),
        endOfDay(this.selectedDate())
      )
      .subscribe(this.diaries);
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
  }

  addDate() {
    if (isToday(this.selectedDate())) {
      return;
    }
    this.selectedDate.set(addDays(this.selectedDate(), 1));
  }

  diaryClicked(diaryId: string) {
    this.router.navigate(['/diary', diaryId]);
  }
}
