export class Diary {
  id: string;
  date: Date;
  title: string;
  content: string;
  created_at: Date;
  updated_at: Date;
  constructor(
    id: string,
    date: Date,
    title: string,
    content: string,
    created_at: Date,
    updated_at: Date
  ) {
    this.id = id;
    this.date = date;
    this.title = title;
    this.content = content;
    this.created_at = created_at;
    this.updated_at = updated_at;
  }
}
