export class Device {
  constructor(
    public name = '',
    public description = '',
    public address = '',
    public can_script = false,
    public scraping = true
  ) {}
}
