import { Component } from '@angular/core';
import { CardComponent } from '../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { FormsModule } from '@angular/forms';
import { PasswordModule } from 'primeng/password';
import { ButtonModule } from 'primeng/button';
import { HttpClient } from '@angular/common/http';
import { RegisterRequestDto, RegisterResponseDto } from '../dtos/register';

@Component({
  selector: 'grn-login',
  standalone: true,
  imports: [
    CardComponent,
    FormsModule,
    InputTextModule,
    PasswordModule,
    ButtonModule,
  ],
  templateUrl: './login.component.html',
  styleUrl: './login.component.scss',
})
export class LoginComponent {
  username: string = '';
  password: string = '';

  constructor(private http: HttpClient) {}

  public login() {
    let a = this.http.post<RegisterResponseDto>(
      'http://localhost:5001/api/login',
      new RegisterRequestDto(this.username, this.password)
    );
    a.subscribe(data => {
      console.log(data);
    });
  }
}
