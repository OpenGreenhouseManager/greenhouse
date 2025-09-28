import { HttpClient } from '@angular/common/http';
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { ButtonModule } from 'primeng/button';
import { DividerModule } from 'primeng/divider';
import { InputTextModule } from 'primeng/inputtext';
import { PasswordModule } from 'primeng/password';
import { CardComponent } from '../card/card.component';
import { RegisterRequestDto, RegisterResponseDto } from '../dtos/register';
import { register } from '../urls/urls';

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
  private http = inject(HttpClient);
  private router = inject(Router);

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
        register,
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
