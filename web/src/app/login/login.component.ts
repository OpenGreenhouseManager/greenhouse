import { Component, signal } from '@angular/core';
import { CardComponent } from '../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { FormsModule } from '@angular/forms';
import { PasswordModule } from 'primeng/password';
import { ButtonModule } from 'primeng/button';
import { HttpClient } from '@angular/common/http';
import { LoginRequestDto, LoginResponseDto } from '../dtos/login';
import { Router } from '@angular/router';
import { apiRegister } from '../urls/urls';

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
  public error = signal(false);

  constructor(
    private http: HttpClient,
    private router: Router
  ) {}

  public login() {
    this.http
      .post<LoginResponseDto>(
        apiRegister,
        new LoginRequestDto(this.username, this.password),
        {
          withCredentials: true,
        }
      )
      .subscribe({
        complete: () => this.router.navigate(['/']),
        error: () => {
          this.password = '';
          this.error.set(true);
        },
      });
  }

  public navRegister() {
    this.router.navigate(['/register']);
  }
}
