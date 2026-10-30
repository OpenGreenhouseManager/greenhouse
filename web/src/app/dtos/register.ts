export class RegisterResponseDto {
  token!: string;
  token_type!: string;

  constructor(token: string, token_type: string) {
    this.token = token;
    this.token_type = token_type;
  }
}

export class RegisterRequestDto {
  username!: string;
  password!: string;
  one_time_token!: string;
  constructor(username: string, password: string, one_time_token: string) {
    this.username = username;
    this.password = password;
    this.one_time_token = one_time_token;
  }
}
