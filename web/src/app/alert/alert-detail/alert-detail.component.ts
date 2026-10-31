import { Location } from '@angular/common';
import { Component, computed, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute } from '@angular/router';
import { ButtonDirective } from 'primeng/button';
import { MessageModule } from 'primeng/message';
import { catchError, of } from 'rxjs';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { AlertAliasService } from '../../services/alert-alias.service';
import { AlertDetailInfoComponent } from '../alert-detail-info/alert-detail-info.component';
import { AlertDetailListComponent } from '../alert-detail-list/alert-detail-list.component';
import { AlertService } from '../services/alert-service';

@Component({
  selector: 'grn-alert-detail',
  imports: [
    NavBarComponent,
    AlertDetailListComponent,
    AlertDetailInfoComponent,
    ButtonDirective,
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
    return this.alertAliasService.getAlias(this.dataSource);
  });
  // undefined while loading, null on error
  alerts = toSignal(
    this.alertService
      .queryAlerts({
        ...(this.identifier ? { identifier: this.identifier } : {}),
        ...(this.dataSource ? { datasource_id: this.dataSource } : {}),
      })
      .pipe(catchError(() => of(null)))
  );

  display(identifier: string): string {
    return identifier.length <= 16
      ? identifier
      : `${identifier.substring(0, 13)}...`;
  }

  goBack() {
    this.location.back();
  }
}
