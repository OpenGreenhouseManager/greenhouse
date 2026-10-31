import { HttpClient } from '@angular/common/http';
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { ButtonDirective } from 'primeng/button';
import { IconField } from 'primeng/iconfield';
import { InputIcon } from 'primeng/inputicon';
import { InputPassword } from 'primeng/inputpassword';
import { InputTextModule } from 'primeng/inputtext';
import { CardComponent } from '../card/card.component';
import type { RegisterRequestDto } from '../dtos/generated/auth_service/RegisterRequestDto';
import type { RegisterResponseDto } from '../dtos/generated/auth_service/RegisterResponseDto';
import { register } from '../urls/urls';

@Component({
  selector: 'grn-register',
  imports: [
    CardComponent,
    FormsModule,
    InputTextModule,
    InputPassword,
    IconField,
    InputIcon,
    ButtonDirective,
  ],
  templateUrl: './register.component.html',
  styleUrl: './register.component.scss',
})
export class RegisterComponent {
  username = '';
  password = signal('');
  passwordRepeat = signal('');
  passwordMasked = signal(true);
  passwordRepeatMasked = signal(true);
  otp = '';

  public error = signal(false);
  public passwordError = signal(false);
  private http = inject(HttpClient);
  private router = inject(Router);

  public register() {
    if (this.password() !== this.passwordRepeat()) {
      this.passwordError.set(true);
      this.password.set('');
      this.passwordRepeat.set('');
      return;
    }
    if (!this.checkPassword(this.password())) {
      this.passwordError.set(true);
      this.password.set('');
      this.passwordRepeat.set('');
      return;
    }
    this.http
      .post<RegisterResponseDto>(
        register,
        {
          username: this.username,
          password: this.password(),
          one_time_token: this.otp,
        } satisfies RegisterRequestDto,
        {
          withCredentials: true,
        }
      )
      .subscribe({
        complete: () => this.router.navigate(['/']),
        error: () => {
          this.password.set('');
          this.passwordRepeat.set('');
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
