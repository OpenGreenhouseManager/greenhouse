import { Component, inject } from '@angular/core';
import { Router, RouterLink, RouterLinkActive } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';

@Component({
  selector: 'grn-nav-bar',
  imports: [RouterLink, RouterLinkActive],
  templateUrl: './nav_bar.component.html',
  styleUrl: './nav_bar.component.scss',
})
export class NavBarComponent {
  private router = inject(Router);
  private cookieService = inject(CookieService);

  signOut() {
    this.cookieService.delete('auth-token');
    this.router.navigate(['/login']);
  }
}
