import { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { AuthGuard } from './_guards/auth.guard';
import { DashboardComponent } from './dashboard/dashboard.component';

export const routes: Routes = [
  { path: 'login', component: LoginComponent },

  // home route protected by auth guard
  {
    path: '',
    component: DashboardComponent,
    canActivate: [AuthGuard],
  },

  // otherwise redirect to home
];
