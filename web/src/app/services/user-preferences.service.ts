import { HttpClient } from '@angular/common/http';
import { inject, Service } from '@angular/core';
import type { Observable } from 'rxjs';
import type { UserPreferencesRequestDto } from '../dtos/generated/auth_service/UserPreferencesRequestDto';
import type { UserPreferencesResponseDto } from '../dtos/generated/auth_service/UserPreferencesResponseDto';
import { userPreferences } from '../urls/urls';

/** Both preferences are stored as JSON-encoded strings in the backend's free-form JSON values. */
export type UserPreferencesDto = UserPreferencesResponseDto &
  UserPreferencesRequestDto & {
    dashboard_preferences: string;
    alert_preferences: string;
  };

@Service()
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
