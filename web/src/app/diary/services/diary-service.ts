import { HttpClient } from '@angular/common/http';
import { diary } from '../../urls/urls';
import { GetDiaryDtoResponse } from '../../dtos/diary';
import { Diary } from '../models/diary';
import { Injectable } from '@angular/core';
import { formatISO, parseISO } from 'date-fns';

const mockDiaries = [
  {
    id: '1',
    date: '2024-09-04T12:13:29+0000',
    title: 'First entry',
    content: 'This is the first entry in the diary',
    created_at: '2024-09-04T12:13:29+0000',
    updated_at: '2024-09-04T12:13:29+0000',
  },
  {
    id: '2',
    date: '2024-09-04T10:13:29+0000',
    title: 'Second entry',
    content: 'This is the second entry in the diary',
    created_at: '2024-09-04T10:13:29+0000',
    updated_at: '2024-09-04T10:13:29+0000',
  },
  {
    id: '3',
    date: '2024-09-03T15:13:29+0000',
    title: 'Third entry',
    content: 'This is the third entry in the diary',
    created_at: '2024-09-03T15:13:29+0000',
    updated_at: '2024-09-03T15:13:29+0000',
  },
];

@Injectable({
  providedIn: 'root',
})
export class DiaryService {
  constructor(private http: HttpClient) {}
  getDiary(id: string): Diary {
    return mockDiaries
      .filter(diary => diary.id === id)
      .map(
        diary =>
          new Diary(
            diary.id,
            parseISO(diary.date),
            diary.title,
            diary.content,
            parseISO(diary.created_at),
            parseISO(diary.updated_at)
          )
      )[0];
  }

  getDiaries(start: Date, end: Date): Diary[] {
    return mockDiaries
      .filter(diary => {
        const diaryDate = new Date(diary.date);
        return diaryDate >= start && diaryDate <= end;
      })
      .map(
        diary =>
          new Diary(
            diary.id,
            parseISO(diary.date),
            diary.title,
            diary.content,
            parseISO(diary.created_at),
            parseISO(diary.updated_at)
          )
      );

    this.http
      .get<GetDiaryDtoResponse>(
        `${diary}?start=${formatISO(start)}&end=${formatISO(end)}`
      )
      .subscribe({
        next: diaries => {
          return diaries.entries.map(
            diary =>
              new Diary(
                diary.id,
                parseISO(diary.date),
                diary.title,
                diary.content,
                parseISO(diary.created_at),
                parseISO(diary.updated_at)
              )
          );
        },
        error: () => {
          return [];
        },
      });
    return [];
  }
  addDiaryEntry() {
    return;
  }
  updateDiaryEntry() {
    return;
  }
  deleteDiaryEntry() {
    return;
  }
}
