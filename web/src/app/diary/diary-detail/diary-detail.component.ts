import { Component, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute, Router } from '@angular/router';
import { ButtonDirective } from 'primeng/button';
import { CardComponent } from '../../card/card.component';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { DiaryService } from '../services/diary-service';

@Component({
  selector: 'grn-diary-detail',
  imports: [NavBarComponent, CardComponent, ButtonDirective],
  templateUrl: './diary-detail.component.html',
  styleUrl: './diary-detail.component.scss',
})
export class DiaryDetailComponent {
  private router = inject(Router);
  private route = inject(ActivatedRoute);
  private diaryService = inject(DiaryService);

  private id = this.route.snapshot.paramMap.get('id');
  diary = toSignal(this.diaryService.getDiary(this.id?.toString() || ''));

  editEntry() {
    this.router.navigate(['diary', this.id, 'edit']);
  }

  goBack() {
    this.router.navigate(['diary']);
  }

  getDate(date: Date | undefined): string {
    if (!date) {
      return '';
    }
    return `${date.toLocaleDateString()} ${date.toLocaleTimeString()}`;
  }
}
