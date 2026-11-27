import { HttpClient } from '@angular/common/http';
import { Component, inject } from '@angular/core';
import { Router, RouterLink, RouterLinkActive } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';
import { logout } from '../urls/urls';

@Component({
  selector: 'grn-nav-bar',
  imports: [RouterLink, RouterLinkActive],
  templateUrl: './nav_bar.component.html',
  styleUrl: './nav_bar.component.scss',
})
export class NavBarComponent {
  private router = inject(Router);
  private cookieService = inject(CookieService);
  private http = inject(HttpClient);

  signOut() {
    const done = () => {
      this.cookieService.delete('auth-token');
      this.router.navigate(['/login']);
    };
    // Clears the HTTP-only refresh cookie, which only the server can do.
    this.http
      .post(logout, null, { withCredentials: true })
      .subscribe({ complete: done, error: done });
  }
}
