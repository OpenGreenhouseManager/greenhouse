import { HttpClient } from '@angular/common/http';
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';
import { ButtonDirective } from 'primeng/button';
import { DialogModule } from 'primeng/dialog';
import { IconField } from 'primeng/iconfield';
import { InputIcon } from 'primeng/inputicon';
import { InputPassword } from 'primeng/inputpassword';
import { InputTextModule } from 'primeng/inputtext';
import { CardComponent } from '../card/card.component';
import type { LoginRequestDto } from '../dtos/generated/auth_service/LoginRequestDto';
import type { LoginResponseDto } from '../dtos/generated/auth_service/LoginResponseDto';
import { login } from '../urls/urls';

@Component({
  selector: 'grn-login',
  imports: [
    CardComponent,
    FormsModule,
    InputTextModule,
    InputPassword,
    IconField,
    InputIcon,
    ButtonDirective,
    DialogModule,
  ],
  templateUrl: './login.component.html',
  styleUrl: './login.component.scss',
})
export class LoginComponent {
  username = '';
  password = signal('');
  passwordMasked = signal(true);
  public error = signal(false);
  public showGuestDialog = signal(false);

  private http = inject(HttpClient);
  private router = inject(Router);
  private cookieService = inject(CookieService);

  private decodeJWT(token: string) {
    try {
      const base64Url = token.split('.')[1];
      if (!base64Url) return null;
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split('')
          .map(c => `%${`00${c.charCodeAt(0).toString(16)}`.slice(-2)}`)
          .join('')
      );
      return JSON.parse(jsonPayload);
    } catch {
      return null;
    }
  }

  private checkForGuestRole(): boolean {
    const token = this.cookieService.get('auth-token');
    if (!token) return false;

    const decoded = this.decodeJWT(token);
    if (!decoded) return false;

    // Check if role field contains 'guest' (case insensitive)
    const role = decoded.role || decoded.roles || '';
    return role.toLowerCase().includes('guest');
  }

  public login() {
    this.http
      .post<LoginResponseDto>(
        login,
        {
          username: this.username,
          password: this.password(),
        } satisfies LoginRequestDto,
        {
          withCredentials: true,
        }
      )
      .subscribe({
        next: response => {
          // Store token in cookie (if not already handled by server)
          if (response.token) {
            this.cookieService.set('auth-token', response.token);
          }

          // Check if user has guest role
          if (this.checkForGuestRole()) {
            this.showGuestDialog.set(true);
          } else {
            this.router.navigate(['/']);
          }
        },
        error: () => {
          this.password.set('');
          this.error.set(true);
        },
      });
  }

  public onGuestDialogClose() {
    this.showGuestDialog.set(false);
    this.router.navigate(['/']);
  }

  public navRegister() {
    this.router.navigate(['/register']);
  }
}
