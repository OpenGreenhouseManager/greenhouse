import type {
  HttpEvent,
  HttpErrorResponse,
  HttpHandlerFn,
} from '@angular/common/http';
import { HttpEventType, HttpRequest } from '@angular/common/http';
import { inject } from '@angular/core';
import { Router } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';
import type { Observable } from 'rxjs';
import { of, throwError } from 'rxjs';
import {
  catchError,
  filter,
  finalize,
  map,
  shareReplay,
  switchMap,
  take,
} from 'rxjs/operators';
import { login, logout, refresh, register } from '../urls/urls';

/** ok: new access token; expired: the session is over; failed: try later. */
type RefreshResult = 'ok' | 'expired' | 'failed';

const sessionUrls = [login, logout, refresh, register];

// One refresh at a time: requests that fail together wait for the same one.
let refreshInFlight: Observable<RefreshResult> | null = null;

function refreshSession(next: HttpHandlerFn): Observable<RefreshResult> {
  refreshInFlight ??= next(
    new HttpRequest('POST', refresh, null, { withCredentials: true })
  ).pipe(
    filter(event => event.type === HttpEventType.Response),
    take(1),
    map((): RefreshResult => 'ok'),
    catchError((error: HttpErrorResponse) =>
      of<RefreshResult>(error.status === 403 ? 'expired' : 'failed')
    ),
    finalize(() => (refreshInFlight = null)),
    shareReplay(1)
  );
  return refreshInFlight;
}

/**
 * Completes once no refresh is running. Sign-out waits for it, so a refresh
 * that was already in flight cannot set fresh cookies after logout, and
 * logout sees the access token naming the session it revokes.
 */
export function refreshSettled(): Observable<unknown> {
  return refreshInFlight ? refreshInFlight.pipe(take(1)) : of(null);
}

/**
 * Access tokens live 10 minutes. A 401 means the token expired (or the role
 * is missing): refresh the session once and retry. A 403 means the session
 * is gone, so drop the cookie and go to the login page.
 */
export function authInterceptor(
  request: HttpRequest<unknown>,
  next: HttpHandlerFn
): Observable<HttpEvent<unknown>> {
  const router = inject(Router);
  const cookieService = inject(CookieService);

  const endSession = () => {
    cookieService.delete('auth-token');
    router.navigate(['/login']);
  };

  const handleError = (error: HttpErrorResponse) => {
    if (error.status === 403) {
      endSession();
    }
    return throwError(() => error);
  };

  return next(request).pipe(
    catchError((error: HttpErrorResponse) => {
      if (error.status !== 401 || sessionUrls.includes(request.url)) {
        return handleError(error);
      }
      return refreshSession(next).pipe(
        switchMap(result => {
          if (result === 'ok') {
            return next(request).pipe(catchError(handleError));
          }
          if (result === 'expired') {
            endSession();
          }
          return throwError(() => error);
        })
      );
    })
  );
}
