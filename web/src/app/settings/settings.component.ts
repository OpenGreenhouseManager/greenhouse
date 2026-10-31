import { HttpClient } from '@angular/common/http';
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ButtonDirective } from 'primeng/button';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputTextModule } from 'primeng/inputtext';
import { CardComponent } from '../card/card.component';
import type { GenerateOneTimeTokenRequestDto } from '../dtos/generated/auth_service/GenerateOneTimeTokenRequestDto';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { otp } from '../urls/urls';

enum SettingsTabs {
  Dashboard,
  Watchdog,
  AdminSettings,
}

@Component({
  selector: 'grn-settings',
  imports: [
    NavBarComponent,
    CardComponent,
    FormsModule,
    InputTextModule,
    ButtonDirective,
    InputGroupModule,
  ],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.scss',
})
export class SettingsComponent {
  currentTab: SettingsTabs = SettingsTabs.Dashboard;
  settingsTabs = SettingsTabs;
  otpUsername = '';
  otp = signal('');
  isCopied = signal(false);

  private http = inject(HttpClient);

  changeTab(tab: SettingsTabs) {
    this.currentTab = tab;
  }

  RequestOtp() {
    this.isCopied.set(false);
    this.http
      .post<string>(
        otp,
        {
          username: this.otpUsername,
        } satisfies GenerateOneTimeTokenRequestDto,
        {
          withCredentials: true,
        }
      )
      .subscribe({
        next: d => {
          this.otp.set(d);
        },
        error: () => {
          this.otp.set('error');
        },
      });
  }

  copyToClipboard() {
    navigator.clipboard.writeText(this.otp());
    this.isCopied.set(true);
  }
}
