import { Component } from '@angular/core';
import { NavBarComponent } from '../../nav_bar/nav_bar.component';
import { CommonModule } from '@angular/common';
import { CardComponent } from '../../card/card.component';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { FormsModule } from '@angular/forms';
import { InputGroupModule } from 'primeng/inputgroup';
import { InputGroupAddonModule } from 'primeng/inputgroupaddon';
import { ActivatedRoute, Router } from '@angular/router';
import { DiaryService } from '../services/diary-service';
import { toSignal } from '@angular/core/rxjs-interop';

@Component({
  selector: 'grn-diary-detail',
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
  templateUrl: './diary-detail.component.html',
  styleUrl: './diary-detail.component.scss',
})
export class DiaryDetailComponent {
  private id = this.route.snapshot.paramMap.get('id');
  diary = toSignal(this.diaryService.getDiary(this.id?.toString() || ''));

  constructor(
    private router: Router,
    private route: ActivatedRoute,
    private diaryService: DiaryService
  ) {}

  editEntry() {
    this.router.navigate(['diary', this.id, 'edit']);
  }

  getDate(date: Date | undefined): string {
    if (!date) {
      return '';
    }
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }
}
