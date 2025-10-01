import { Location } from '@angular/common';
import { Component, computed, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';
import { ButtonModule } from 'primeng/button';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { InputTextModule } from 'primeng/inputtext';
import { MessageModule } from 'primeng/message';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { AlertAliasService } from '../../services/alert-alias.service';
import { AlertDetailInfoComponent } from '../alert-detail-info/alert-detail-info.component';
import { AlertDetailListComponent } from '../alert-detail-list/alert-detail-list.component';
import { AlertService } from '../services/alert-service';

@Component({
  selector: 'grn-alert-detail',
  standalone: true,
  imports: [
    NavBarComponent,
    AlertDetailListComponent,
    AlertDetailInfoComponent,
    FormsModule,
    InputTextModule,
    ButtonModule,
    InputGroupModule,
    InputGroupAddonModule,
    MessageModule,
  ],
  templateUrl: './alert-detail.component.html',
  styleUrl: './alert-detail.component.scss',
})
export class AlertDetailComponent {
  private route = inject(ActivatedRoute);
  private alertService = inject(AlertService);
  private alertAliasService = inject(AlertAliasService);
  private location = inject(Location);

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
    return this.alertAliasService.getAliasSync(identifier);
  }

  goBack() {
    this.location.back();
  }
}
