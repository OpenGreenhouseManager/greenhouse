import {
  HttpClient,
  provideHttpClient,
  withInterceptors,
} from '@angular/common/http';
import {
  HttpTestingController,
  provideHttpClientTesting,
} from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { Router, provideRouter } from '@angular/router';
import { CookieService } from 'ngx-cookie-service';
import { refresh } from '../urls/urls';
import { authInterceptor, refreshSettled } from './auth.interceptor';

describe('authInterceptor', () => {
  let http: HttpClient;
  let backend: HttpTestingController;
  let router: Router;
  let cookies: CookieService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideRouter([]),
        provideHttpClient(withInterceptors([authInterceptor])),
        provideHttpClientTesting(),
      ],
    });
    http = TestBed.inject(HttpClient);
    backend = TestBed.inject(HttpTestingController);
    router = TestBed.inject(Router);
    cookies = TestBed.inject(CookieService);
    vi.spyOn(router, 'navigate').mockResolvedValue(true);
    vi.spyOn(cookies, 'delete');
  });

  afterEach(() => backend.verify());

  it('refreshes once on 401 and retries the request', () => {
    let body: unknown;
    http.get('/api/diary').subscribe(b => (body = b));

    backend
      .expectOne('/api/diary')
      .flush(null, { status: 401, statusText: 'Unauthorized' });
    backend.expectOne(refresh).flush({ token: 't', token_type: 'Bearer' });
    backend.expectOne('/api/diary').flush({ ok: true });

    expect(body).toEqual({ ok: true });
    expect(router.navigate).not.toHaveBeenCalled();
  });

  it('shares one refresh between concurrent 401s', () => {
    http.get('/api/a').subscribe();
    http.get('/api/b').subscribe();

    backend
      .expectOne('/api/a')
      .flush(null, { status: 401, statusText: 'Unauthorized' });
    backend
      .expectOne('/api/b')
      .flush(null, { status: 401, statusText: 'Unauthorized' });
    backend.expectOne(refresh).flush({ token: 't', token_type: 'Bearer' });
    backend.expectOne('/api/a').flush({});
    backend.expectOne('/api/b').flush({});
  });

  it('logs out when the session is gone', () => {
    let status = 0;
    http.get('/api/diary').subscribe({ error: e => (status = e.status) });

    backend
      .expectOne('/api/diary')
      .flush(null, { status: 401, statusText: 'Unauthorized' });
    backend
      .expectOne(refresh)
      .flush(null, { status: 403, statusText: 'Forbidden' });

    expect(status).toBe(401);
    expect(cookies.delete).toHaveBeenCalledWith('auth-token');
    expect(router.navigate).toHaveBeenCalledWith(['/login']);
  });

  it('does not retry a second time', () => {
    let status = 0;
    http.get('/api/settings').subscribe({ error: e => (status = e.status) });

    backend
      .expectOne('/api/settings')
      .flush(null, { status: 401, statusText: 'Unauthorized' });
    backend.expectOne(refresh).flush({ token: 't', token_type: 'Bearer' });
    backend
      .expectOne('/api/settings')
      .flush(null, { status: 401, statusText: 'Unauthorized' });

    expect(status).toBe(401);
    expect(router.navigate).not.toHaveBeenCalled();
  });

  it('lets sign-out wait for a refresh in flight', () => {
    http.get('/api/diary').subscribe();
    backend
      .expectOne('/api/diary')
      .flush(null, { status: 401, statusText: 'Unauthorized' });

    let settled = false;
    refreshSettled().subscribe(() => (settled = true));
    expect(settled).toBe(false);

    backend.expectOne(refresh).flush({ token: 't', token_type: 'Bearer' });
    expect(settled).toBe(true);
    backend.expectOne('/api/diary').flush({});

    let idle = false;
    refreshSettled().subscribe(() => (idle = true));
    expect(idle).toBe(true);
  });

  it('logs out on 403 without refreshing', () => {
    http.get('/api/diary').subscribe({ error: () => undefined });

    backend
      .expectOne('/api/diary')
      .flush(null, { status: 403, statusText: 'Forbidden' });

    expect(router.navigate).toHaveBeenCalledWith(['/login']);
  });
});
