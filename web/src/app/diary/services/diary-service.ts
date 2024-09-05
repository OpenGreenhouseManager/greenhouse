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
            parseISO(diary.date),
            diary.title,
            diary.content,
            parseISO(diary.created_at),
            parseISO(diary.updated_at)
          )
      )[0];
  }

  getDiaries(start: Date, end: Date): Map<string, Diary> {
    const a = mockDiaries.filter(diary => {
      const date = parseISO(diary.date);
      return date >= start && date <= end;
    });

    const b = new Map<string, Diary>();
    for (let i = 0; i < a.length; i++) {
      const diary = a[i];
      b.set(
        diary.id,
        new Diary(
          parseISO(diary.date),
          diary.title,
          diary.content,
          parseISO(diary.created_at),
          parseISO(diary.updated_at)
        )
      );
    }
    return b;
  }
  addDiaryEntry(diary: Diary) {
    return;
  }
  updateDiaryEntry(id: string, diary: Diary) {
    return;
  }
  deleteDiaryEntry(id: string) {
    return;
  }
}
