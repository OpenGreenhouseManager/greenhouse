import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';

@Component({
  selector: 'grn-root',
  imports: [RouterOutlet],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent {
  public str: string = $localize`:@@HOME-TEXT1:Add some content to get startedd!`;
  public str2: string = $localize`:@@HOME-TEXT2:this is some more localization!`;
  title = 'greenhouse_web';
}
