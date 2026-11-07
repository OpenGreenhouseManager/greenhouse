import { booleanAttribute, Directive, input, output } from '@angular/core';

/**
 * Put on a widget whose template contains a `grn-card` (e.g.
 * `<grn-graph grnMaximizable (maximizeToggle)="...">`) to give that card a
 * maximize / restore button. The card finds this directive through DI, so
 * the widget itself needs no extra inputs. The host decides what maximizing
 * means; the directive only carries the state and the click.
 */
@Directive({ selector: '[grnMaximizable]' })
export class MaximizableDirective {
  /** Set to false to hide the button, e.g. while the dashboard is edited. */
  enabled = input(true, {
    alias: 'grnMaximizable',
    transform: booleanAttribute,
  });
  /** Whether the widget is shown maximized, flips the button to restore. */
  maximized = input(false);
  maximizeToggle = output();
}
