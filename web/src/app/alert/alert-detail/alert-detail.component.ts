import { Component, computed } from '@angular/core';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { CommonModule } from '@angular/common';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { ActivatedRoute, Router } from '@angular/router';
import { AlertService } from '../services/alert-service';
import { toSignal } from '@angular/core/rxjs-interop';
import { AlertDetailInfoComponent } from '../alert-detail-info/alert-detail-info.component';
import { AlertDetailListComponent } from '../alert-detail-list/alert-detail-list.component';

@Component({
  selector: 'grn-alert-detail',
  standalone: true,
  imports: [
    NavBarComponent,
    CommonModule,
    AlertDetailListComponent,
    AlertDetailInfoComponent,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    InputGroupAddonModule,
  ],
  templateUrl: './alert-detail.component.html',
  styleUrl: './alert-detail.component.scss',
})
export class AlertDetailComponent {
  identifier = this.route.snapshot.paramMap.get('identifier');
  dataSource = this.route.snapshot.paramMap.get('data-source');
  alias = computed(() => {
    if (!this.identifier || !this.dataSource) {
      return null;
    }
    return this.searchForAlias(this.dataSource);
  });
  alerts = toSignal(
    this.alertService.queryAlerts({
      identifier: this.identifier || undefined,
      datasource_id: this.dataSource || undefined,
    })
  );

  constructor(
    private router: Router,
    private route: ActivatedRoute,
    private alertService: AlertService
  ) {}

  getDate(date: Date | undefined): string {
    if (!date) {
      return '';
    }
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }

  display(identifier: string): string {
    return identifier.length <= 16
      ? identifier
      : `${identifier.substring(0, 13)}...`;
  }

  searchForAlias(identifier: string): string | null {
    return localStorage.getItem(`alias_${identifier}`);
  }
}
