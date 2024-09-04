export class GetDiaryDtoResponse {
  entries!: DiaryEntryDto[];
}

export class DiaryEntryDto {
  id!: string;
  date!: string;
  title!: string;
  content!: string;
  created_at!: string;
  updated_at!: string;
}
