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
import { firstValueFrom } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class DiaryService {
  constructor(private http: HttpClient) {}
  async getDiary(id: string): Promise<Diary | null> {
    const response = await firstValueFrom(
      this.http.get<DiaryEntryDtoResponse>(`${diary}/${id}`)
    );

    if (response) {
      return new Diary(
        parseISO(response.date),
        response.title,
        response.content,
        parseISO(response.created_at),
        parseISO(response.updated_at)
      );
    }

    return null;
  }

  async getDiaries(start: Date, end: Date): Promise<Map<string, Diary>> {
    const response = await firstValueFrom(
      this.http.get<GetDiaryDtoResponse>(diary, {
        params: {
          start: start.toISOString(),
          end: end.toISOString(),
        },
      })
    );

    const diaries = new Map<string, Diary>();
    response.entries.forEach(entry => {
      diaries.set(
        entry.id,
        new Diary(
          parseISO(entry.date),
          entry.title,
          entry.content,
          parseISO(entry.created_at),
          parseISO(entry.updated_at)
        )
      );
    });

    return diaries;
  }

  async addDiaryEntry(newDiary: Diary): Promise<Diary> {
    const d = await firstValueFrom(
      this.http.post<PostDiaryEntryDtoResponse>(diary, {
        date: formatISO(newDiary.date),
        title: newDiary.title,
        content: newDiary.content,
      } as PostDiaryEntryDtoRequest)
    );

    return new Diary(
      parseISO(d.date),
      d.title,
      d.content,
      parseISO(d.created_at),
      parseISO(d.updated_at)
    );
  }

  async updateDiaryEntry(id: string, newDiary: Diary): Promise<Diary> {
    const d = await firstValueFrom(
      this.http.put<PutDiaryEntryDtoResponse>(`${diary}/${id}`, {
        date: formatISO(newDiary.date),
        title: newDiary.title,
        content: newDiary.content,
      } as PutDiaryEntryDtoRequest)
    );

    return new Diary(
      parseISO(d.date),
      d.title,
      d.content,
      parseISO(d.created_at),
      parseISO(d.updated_at)
    );
  }

  async deleteDiaryEntry(id: string) {
    await firstValueFrom(this.http.delete(`${diary}/${id}`));
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
