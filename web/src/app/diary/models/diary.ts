export class Diary {
  date: Date;
  title: string;
  content: string;
  created_at: Date;
  updated_at: Date;
  constructor(
    date: Date,
    title: string,
    content: string,
    created_at: Date,
    updated_at: Date
  ) {
    this.date = date;
    this.title = title;
    this.content = content;
    this.created_at = created_at;
    this.updated_at = updated_at;
  }
}
