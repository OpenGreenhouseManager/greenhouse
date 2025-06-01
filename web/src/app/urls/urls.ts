import { environment } from '../../environment/environment';

export const register = environment.baseUrl + '/api/register';
export const login = environment.baseUrl + '/api/login';
export const otp =
  environment.baseUrl + '/api/settings/generate_one_time_token';
export const diary = environment.baseUrl + '/api/diary';
export const alert = environment.baseUrl + '/api/alert';
