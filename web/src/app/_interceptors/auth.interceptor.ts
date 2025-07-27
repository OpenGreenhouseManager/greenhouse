import {
  HttpRequest,
  HttpHandlerFn,
  HttpEvent,
  HttpErrorResponse,
} from '@angular/common/http';
import { Observable, throwError } from 'rxjs';
import { catchError } from 'rxjs/operators';
import { inject } from '@angular/core';
import { Router } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';

export function authInterceptor(
  request: HttpRequest<unknown>,
  next: HttpHandlerFn
): Observable<HttpEvent<unknown>> {
  const router = inject(Router);
  const cookieService = inject(CookieService);

  return next(request).pipe(
    catchError((error: HttpErrorResponse) => {
      if (error.status === 403) {
        cookieService.delete('auth-token');
        router.navigate(['/login']);
      }
      
      return throwError(() => error);
    })
  );
} 