export class GenerateOneTimeTokenRequestDto {
  username!: string;

  constructor(username: string) {
    this.username = username;
  }
}

export class GenerateOneTimeTokenResponseDto {
  token!: string;

  constructor(token: string) {
    this.token = token;
  }
}
