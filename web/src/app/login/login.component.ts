import { Component } from '@angular/core';
import { CardComponent } from '../card/card.component';

@Component({
  selector: 'grn-login',
  standalone: true,
  imports: [CardComponent],
  templateUrl: './login.component.html',
  styleUrl: './login.component.scss',
})
export class LoginComponent {}
