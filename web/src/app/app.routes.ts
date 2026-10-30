import type { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { authGuard } from './_guards/auth.guard';
import { DashboardComponent } from './dashboard/dashboard.component';
import { RegisterComponent } from './register/register.component';
import { SettingsComponent } from './settings/settings.component';
import { DiaryOverviewComponent } from './diary/diary-overview/diary-overview.component';
import { DiaryDetailComponent } from './diary/diary-detail/diary-detail.component';
import { DiaryEditComponent } from './diary/diary-edit/diary-edit.component';
import { AlertOverviewComponent } from './alert/alert-overview/alert-overview.component';
import { AlertDetailComponent } from './alert/alert-detail/alert-detail.component';
import { DeviceOverviewComponent } from './device/device-overview/device-overview.component';
import { DeviceDetailComponent } from './device/device-detail/device-detail.component';
import { DeviceEditComponent } from './device/device-edit/device-edit.component';

export const routes: Routes = [
  { path: 'login', component: LoginComponent },
  { path: 'register', component: RegisterComponent },

  // home route protected by auth guard
  {
    path: '',
    component: DashboardComponent,
    canActivate: [authGuard],
  },
  {
    path: 'smart_devices',
    component: DeviceOverviewComponent,
    canActivate: [authGuard],
  },
  {
    path: 'smart_devices/add',
    component: DeviceEditComponent,
    canActivate: [authGuard],
  },
  {
    path: 'smart_devices/:id',
    component: DeviceDetailComponent,
    canActivate: [authGuard],
  },
  {
    path: 'smart_devices/:id/edit',
    component: DeviceEditComponent,
    canActivate: [authGuard],
  },
  {
    path: 'alerts',
    component: AlertOverviewComponent,
    canActivate: [authGuard],
  },

  {
    path: 'alert/:identifier/:data-source',
    component: AlertDetailComponent,
    canActivate: [authGuard],
  },
  {
    path: 'diary',
    component: DiaryOverviewComponent,
    canActivate: [authGuard],
  },
  {
    path: 'diary/add',
    component: DiaryEditComponent,
    canActivate: [authGuard],
  },
  {
    path: 'diary/:id',
    component: DiaryDetailComponent,
    canActivate: [authGuard],
  },
  {
    path: 'diary/:id/edit',
    component: DiaryEditComponent,
    canActivate: [authGuard],
  },
  {
    path: 'scripts',
    component: DashboardComponent,
    canActivate: [authGuard],
  },
  {
    path: 'settings',
    component: SettingsComponent,
    canActivate: [authGuard],
  },
  {
    path: 'logout',
    component: DashboardComponent,
    canActivate: [authGuard],
  },

  // otherwise redirect to home
];
