import { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { AuthGuard } from './_guards/auth.guard';
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
    canActivate: [AuthGuard],
  },
  {
    path: 'smart_devices',
    component: DeviceOverviewComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'smart_devices/add',
    component: DeviceEditComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'smart_devices/:id',
    component: DeviceDetailComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'smart_devices/:id/edit',
    component: DeviceEditComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'alerts',
    component: AlertOverviewComponent,
    canActivate: [AuthGuard],
  },

  {
    path: 'alert/:identifier/:data-source',
    component: AlertDetailComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'diary',
    component: DiaryOverviewComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'diary/add',
    component: DiaryEditComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'diary/:id',
    component: DiaryDetailComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'diary/:id/edit',
    component: DiaryEditComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'scripts',
    component: DashboardComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'settings',
    component: SettingsComponent,
    canActivate: [AuthGuard],
  },
  {
    path: 'logout',
    component: DashboardComponent,
    canActivate: [AuthGuard],
  },

  // otherwise redirect to home
];
