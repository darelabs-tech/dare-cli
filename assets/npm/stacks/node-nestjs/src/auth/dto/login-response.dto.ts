import { ApiProperty } from '@nestjs/swagger';

export class LoginResponseDto {
  @ApiProperty()
  accessToken!: string;

  @ApiProperty({ description: 'Seconds until token expiry' })
  expiresIn!: number;
}
