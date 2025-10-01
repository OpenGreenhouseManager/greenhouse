import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { UserPreferencesDto } from '../dtos/user-preferences';
import { userPreferences } from '../urls/urls';

@Injectable({
  providedIn: 'root',
})
export class UserPreferencesService {
  private http = inject(HttpClient);

  /**
   * Get user preferences from the backend
   */
  getUserPreferences(): Observable<UserPreferencesDto> {
    return this.http.get<UserPreferencesDto>(userPreferences, {
      withCredentials: true,
    });
  }

  /**
   * Update user preferences on the backend
   */
  updateUserPreferences(
    preferences: UserPreferencesDto
  ): Observable<UserPreferencesDto> {
    return this.http.post<UserPreferencesDto>(userPreferences, preferences, {
      withCredentials: true,
    });
  }
}
