export class Device {
  constructor(
    public name: string = '',
    public description: string = '',
    public address: string = '',
    public can_script: boolean = false,
    public scraping: boolean = true
  ) {}
}
