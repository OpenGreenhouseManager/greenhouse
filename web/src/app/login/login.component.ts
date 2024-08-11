import { Component } from '@angular/core';
import { CardComponent } from '../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { FormsModule } from '@angular/forms';
import { PasswordModule } from 'primeng/password';
import { ButtonModule } from 'primeng/button';
import { HttpClient } from '@angular/common/http';
import { LoginRequestDto, LoginResponseDto } from '../dtos/login';
import { CookieService } from 'ngx-cookie-service';

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

  constructor(private http: HttpClient, private cookieService: CookieService) {}

  public login() {
    let a = this.http.post<LoginResponseDto>(
      'http://localhost:5001/api/login',
      new LoginRequestDto(this.username, this.password),
      {
        withCredentials: true
      }
    );
    a.subscribe(data => {
      //this.cookieService.set('auth-token', data.token)
      console.log(data);
    });
  }
}
