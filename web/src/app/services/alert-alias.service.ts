import { inject, Service, signal } from '@angular/core';
import type { Observable } from 'rxjs';
import { map, switchMap } from 'rxjs';
import { UserPreferencesService } from './user-preferences.service';

@Service()
export class AlertAliasService {
  private userPreferencesService = inject(UserPreferencesService);
  private aliases = signal<Record<string, string>>({});

  constructor() {
    this.loadAliases();
  }

  /**
   * Load aliases from backend preferences
   */
  private loadAliases(): void {
    this.userPreferencesService.getUserPreferences().subscribe({
      next: preferences => {
        try {
          this.aliases.set(
            preferences.alert_preferences
              ? JSON.parse(preferences.alert_preferences)
              : {}
          );
        } catch {
          this.aliases.set({});
        }
      },
      error: _ => {
        this.aliases.set({});
      },
    });
  }

  /**
   * Get alias for a specific identifier. Reads a signal, so callers in a
   * template or `computed` update once aliases load or change.
   */
  getAlias(identifier: string): string | null {
    return this.aliases()[identifier] || null;
  }

  /**
   * Set or update an alias for an identifier
   */
  setAlias(identifier: string, alias: string): Observable<void> {
    const currentAliases = { ...this.aliases() };

    if (alias.trim() === '') {
      delete currentAliases[identifier];
    } else {
      currentAliases[identifier] = alias;
    }

    return this.saveAliases(currentAliases);
  }

  /**
   * Remove an alias for an identifier
   */
  removeAlias(identifier: string): Observable<void> {
    const currentAliases = { ...this.aliases() };
    delete currentAliases[identifier];
    return this.saveAliases(currentAliases);
  }

  /**
   * Save aliases to backend preferences
   */
  private saveAliases(aliases: Record<string, string>): Observable<void> {
    return this.userPreferencesService.getUserPreferences().pipe(
      switchMap(currentPreferences => {
        const updatedPreferences = {
          ...currentPreferences,
          alert_preferences: JSON.stringify(aliases),
        };

        return this.userPreferencesService.updateUserPreferences(
          updatedPreferences
        );
      }),
      map(() => {
        this.aliases.set(aliases);
      })
    );
  }
}
