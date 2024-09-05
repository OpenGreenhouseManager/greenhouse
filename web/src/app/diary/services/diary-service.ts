import { HttpClient } from '@angular/common/http';
import { diary } from '../../urls/urls';
import {
  DiaryEntryDtoResponse,
  GetDiaryDtoResponse,
  PostDiaryEntryDtoRequest,
  PostDiaryEntryDtoResponse,
  PutDiaryEntryDtoRequest,
  PutDiaryEntryDtoResponse,
} from '../../dtos/diary';
import { Diary } from '../models/diary';
import { Injectable } from '@angular/core';
import { formatISO, parseISO } from 'date-fns';
import { map, Observable } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class DiaryService {
  constructor(private http: HttpClient) {}
  getDiary(id: string): Observable<Diary> {
    return this.http
      .get<DiaryEntryDtoResponse>(`${diary}/${id}`)
      .pipe(
        map(
          x =>
            new Diary(
              parseISO(x.date),
              x.title,
              x.content,
              parseISO(x.created_at),
              parseISO(x.updated_at)
            )
        )
      );
  }

  getDiaries(start: Date, end: Date): Observable<Map<string, Diary>> {
    return this.http
      .get<GetDiaryDtoResponse>(diary, {
        params: {
          start: start.toISOString(),
          end: end.toISOString(),
        },
      })
      .pipe(
        map(x => {
          const a = new Map<string, Diary>();
          for (let i = 0; i < x.entries.length; i++) {
            const diary = x.entries[i];
            a.set(
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
          return a;
        })
      );
  }

  addDiaryEntry(newDiary: Diary): Observable<Diary> {
    return this.http
      .post<PostDiaryEntryDtoResponse>(diary, {
        date: formatISO(newDiary.date),
        title: newDiary.title,
        content: newDiary.content,
      } as PostDiaryEntryDtoRequest)
      .pipe(
        map(x => {
          return new Diary(
            parseISO(x.date),
            x.title,
            x.content,
            parseISO(x.created_at),
            parseISO(x.updated_at)
          );
        })
      );
  }

  updateDiaryEntry(id: string, newDiary: Diary): Observable<Diary> {
    return this.http
      .put<PutDiaryEntryDtoResponse>(`${diary}/${id}`, {
        date: formatISO(newDiary.date),
        title: newDiary.title,
        content: newDiary.content,
      } as PutDiaryEntryDtoRequest)
      .pipe(
        map(x => {
          return new Diary(
            parseISO(x.date),
            x.title,
            x.content,
            parseISO(x.created_at),
            parseISO(x.updated_at)
          );
        })
      );
  }

  deleteDiaryEntry(id: string): Observable<void> {
    return this.http.delete<void>(`${diary}/${id}`);
  }
}

export class MockSrevice {
  static mockDiaries = [
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

  static getDiary(id: string): Diary {
    return this.mockDiaries
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

  static getDiaries(start: Date, end: Date): Map<string, Diary> {
    const a = MockSrevice.mockDiaries.filter(diary => {
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

  static addDiaryEntry(diary: Diary) {
    const random_id = Math.floor(Math.random() * 1000).toString();
    MockSrevice.mockDiaries.push({
      id: random_id,
      date: formatISO(diary.date),
      title: diary.title,
      content: diary.content,
      created_at: formatISO(diary.created_at),
      updated_at: formatISO(diary.updated_at),
    });
  }

  static updateDiaryEntry(id: string, diary: Diary) {
    const e = MockSrevice.mockDiaries.find(d => d.id === id);
    if (e) {
      e.date = formatISO(diary.date);
      e.title = diary.title;
      e.content = diary.content;
      e.updated_at = formatISO(diary.updated_at);
    }
  }

  static deleteDiaryEntry(id: string) {
    MockSrevice.mockDiaries = MockSrevice.mockDiaries.filter(
      diary => diary.id !== id
    );
  }
}
