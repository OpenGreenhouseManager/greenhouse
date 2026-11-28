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
    // The server revokes the session of the HTTP-only refresh cookie and
    // clears it. Local state is only dropped once that succeeded, so a failed
    // sign-out can be retried instead of leaving the session alive.
    this.http.post(logout, null, { withCredentials: true }).subscribe({
      complete: () => {
        this.cookieService.delete('auth-token');
        this.router.navigate(['/login']);
      },
    });
  }
}
