import { inject, Injectable } from '@angular/core';
import { BehaviorSubject, map, Observable, switchMap } from 'rxjs';
import { UserPreferencesService } from './user-preferences.service';

@Injectable({
  providedIn: 'root',
})
export class AlertAliasService {
  private userPreferencesService = inject(UserPreferencesService);
  private aliasesSubject = new BehaviorSubject<Record<string, string>>({});

  // Observable of current aliases
  public aliases$ = this.aliasesSubject.asObservable();

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
          const aliases = preferences.alert_preferences
            ? JSON.parse(preferences.alert_preferences)
            : {};
          this.aliasesSubject.next(aliases);
        } catch (error) {
          console.error('Error parsing alert preferences:', error);
          this.aliasesSubject.next({});
        }
      },
      error: error => {
        console.error('Error loading alert preferences:', error);
        this.aliasesSubject.next({});
      },
    });
  }

  /**
   * Get alias for a specific identifier
   */
  getAlias(identifier: string): Observable<string | null> {
    return this.aliases$.pipe(map(aliases => aliases[identifier] || null));
  }

  /**
   * Get alias synchronously (for immediate use)
   */
  getAliasSync(identifier: string): string | null {
    const currentAliases = this.aliasesSubject.value;
    return currentAliases[identifier] || null;
  }

  /**
   * Set or update an alias for an identifier
   */
  setAlias(identifier: string, alias: string): Observable<void> {
    const currentAliases = { ...this.aliasesSubject.value };

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
    const currentAliases = { ...this.aliasesSubject.value };
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
        this.aliasesSubject.next(aliases);
        console.log('Alert aliases saved successfully');
      })
    );
  }
}
