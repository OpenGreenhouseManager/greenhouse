export class GetDiaryDtoResponse {
  entries!: DiaryEntryDtoResponse[];
}

export class DiaryEntryDtoResponse {
  id!: string;
  date!: string;
  title!: string;
  content!: string;
  created_at!: string;
  updated_at!: string;
}

export class PostDiaryEntryDtoRequest {
  date!: string;
  title!: string;
  content!: string;
}

export class PostDiaryEntryDtoResponse {
  id!: string;
  date!: string;
  title!: string;
  content!: string;
  created_at!: string;
  updated_at!: string;
}

export class PutDiaryEntryDtoRequest {
  date!: string;
  title!: string;
  content!: string;
}

export class PutDiaryEntryDtoResponse {
  id!: string;
  date!: string;
  title!: string;
  content!: string;
  created_at!: string;
  updated_at!: string;
}
