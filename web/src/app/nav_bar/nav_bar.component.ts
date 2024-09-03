import { Component } from '@angular/core';
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
  constructor(
    private router: Router,
    private cookieService: CookieService
  ) {}

  signOut() {
    this.cookieService.delete('auth-token');
    this.router.navigate(['/login']);
  }
}
