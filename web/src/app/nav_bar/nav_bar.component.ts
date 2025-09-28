import { Component, inject } from '@angular/core';
import { Router, RouterModule } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';

@Component({
  selector: 'grn-nav-bar',
  standalone: true,
  imports: [RouterModule],
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
