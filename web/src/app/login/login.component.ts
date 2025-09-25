import { HttpClient } from '@angular/common/http';
import { Component, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';
import { ButtonModule } from 'primeng/button';
import { DialogModule } from 'primeng/dialog';
import { InputTextModule } from 'primeng/inputtext';
import { PasswordModule } from 'primeng/password';
import { CardComponent } from '../card/card.component';
import { LoginRequestDto, LoginResponseDto } from '../dtos/login';
import { login } from '../urls/urls';

@Component({
  selector: 'grn-login',
  standalone: true,
  imports: [
    CardComponent,
    FormsModule,
    InputTextModule,
    PasswordModule,
    ButtonModule,
    DialogModule,
  ],
  templateUrl: './login.component.html',
  styleUrl: './login.component.scss',
})
export class LoginComponent {
  username: string = '';
  password: string = '';
  public error = signal(false);
  public showGuestDialog = signal(false);

  constructor(
    private http: HttpClient,
    private router: Router,
    private cookieService: CookieService
  ) {}

  private decodeJWT(token: string): any {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split('')
          .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
          .join('')
      );
      return JSON.parse(jsonPayload);
    } catch (error) {
      console.error('Error decoding JWT:', error);
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
        new LoginRequestDto(this.username, this.password),
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
          this.password = '';
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
