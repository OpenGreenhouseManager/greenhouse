import { Component } from '@angular/core';
import { NavBarComponent } from '../nav_bar/nav_bar.component';
import { CommonModule } from '@angular/common';
import { CardComponent } from '../card/card.component';
import { HttpClient } from '@angular/common/http';
import { GenerateOneTimeTokenRequestDto } from '../dtos/generate_one_time_token_request';
import { otp } from '../urls/urls';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';

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

  constructor(private http: HttpClient) {}

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
