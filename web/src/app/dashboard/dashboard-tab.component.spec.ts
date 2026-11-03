import { TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ConfirmationService } from 'primeng/api';
import type { DashboardItem } from './dashboard-tab.component';
import { DashboardTabComponent } from './dashboard-tab.component';

function value(name: string, x: number, y: number): DashboardItem {
  return {
    cols: 2,
    rows: 2,
    x,
    y,
    type: 'latestValue',
    options: { name, deviceId: 'device' },
  };
}

describe('DashboardTabComponent', () => {
  function create(compact: boolean) {
    vi.spyOn(window, 'matchMedia').mockReturnValue({
      matches: compact,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    } as unknown as MediaQueryList);
    TestBed.configureTestingModule({
      imports: [DashboardTabComponent],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting(),
        ConfirmationService,
      ],
    });
    const fixture = TestBed.createComponent(DashboardTabComponent);
    fixture.componentRef.setInput('dashboard', [
      value('right', 4, 0),
      value('below', 0, 2),
      value('left', 0, 0),
    ]);
    fixture.componentRef.setInput('editMode', false);
    fixture.detectChanges();
    return fixture;
  }

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('only drags and resizes in edit mode', () => {
    const fixture = create(false);
    const component = fixture.componentInstance;
    expect(component.options().draggable?.enabled).toBe(false);
    expect(component.options().resizable?.enabled).toBe(false);

    fixture.componentRef.setInput('editMode', true);
    expect(component.options().draggable?.enabled).toBe(true);
    expect(component.options().resizable?.enabled).toBe(true);
  });

  it('keeps the stored order in the grid layout', () => {
    const component = create(false).componentInstance;
    expect(component.items().map(i => i.options.name)).toEqual([
      'right',
      'below',
      'left',
    ]);
  });

  it('stacks widgets in reading order on small screens', () => {
    const component = create(true).componentInstance;
    expect(component.items().map(i => i.options.name)).toEqual([
      'left',
      'right',
      'below',
    ]);
  });
});
