import { Component, signal } from '@angular/core';
import { CardComponent } from '../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { FormsModule } from '@angular/forms';
import { PasswordModule } from 'primeng/password';
import { ButtonModule } from 'primeng/button';
import { HttpClient } from '@angular/common/http';
import { RegisterRequestDto, RegisterResponseDto } from '../dtos/register';
import { Router } from '@angular/router';
import { apiRegister } from '../urls/urls';
import { DividerModule } from 'primeng/divider';

@Component({
  selector: 'grn-register',
  standalone: true,
  imports: [
    CardComponent,
    FormsModule,
    InputTextModule,
    PasswordModule,
    ButtonModule,
    DividerModule,
  ],
  templateUrl: './register.component.html',
  styleUrl: './register.component.scss',
})
export class RegisterComponent {
  username: string = '';
  password: string = '';
  passwordRepeat: string = '';
  otp: string = '';

  public error = signal(false);
  public passwordError = signal(false);
  constructor(
    private http: HttpClient,
    private router: Router
  ) {}

  public register() {
    if (this.password !== this.passwordRepeat) {
      this.passwordError.set(true);
      this.password = '';
      this.passwordRepeat = '';
      return;
    }
    if (!this.checkPassword(this.password)) {
      this.passwordError.set(true);
      this.password = '';
      this.passwordRepeat = '';
      return;
    }
    this.http
      .post<RegisterResponseDto>(
        apiRegister,
        new RegisterRequestDto(this.username, this.password, this.otp),
        {
          withCredentials: true,
        }
      )
      .subscribe({
        complete: () => this.router.navigate(['/']),
        error: () => {
          this.password = '';
          this.passwordRepeat = '';
          this.error.set(true);
        },
      });
  }

  private checkPassword(password: string) {
    const regularExpression =
      /^(?=.*[0-9])(?=.*[!@#$%^&*])[a-zA-Z0-9!@#$%^&*]{8,}$/;

    return password.match(regularExpression);
  }

  public navLogin() {
    this.router.navigate(['/login']);
  }
}
