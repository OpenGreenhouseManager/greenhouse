import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ButtonModule } from 'primeng/button';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { InputTextModule } from 'primeng/inputtext';
import { CardComponent } from '../card/card.component';
import { GenerateOneTimeTokenRequestDto } from '../dtos/generate_one_time_token_request';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { otp } from '../urls/urls';

enum SettingsTabs {
  Dashboard,
  Watchdog,
  AdminSettings,
}

@Component({
  selector: 'grn-settings',
  standalone: true,
  imports: [
    NavBarComponent,
    CommonModule,
    CardComponent,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    InputGroupAddonModule,
  ],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.scss',
})
export class SettingsComponent {
  currentTab: SettingsTabs = SettingsTabs.Dashboard;
  settingsTabs = SettingsTabs;
  otpUsername: string = '';
  otp: string = '';
  isCopied: boolean = false;

  private http = inject(HttpClient);

  changeTab(tab: SettingsTabs) {
    this.currentTab = tab;
  }

  RequestOtp() {
    this.isCopied = false;
    this.http
      .post<string>(otp, new GenerateOneTimeTokenRequestDto(this.otpUsername), {
        withCredentials: true,
      })
      .subscribe({
        next: d => {
          this.otp = d;
        },
        error: () => {
          this.otp = 'error';
        },
      });
  }

  copyToClipboard() {
    navigator.clipboard.writeText(this.otp);
    this.isCopied = true;
  }
}
