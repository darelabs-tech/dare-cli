import { ApiProperty } from '@nestjs/swagger';
import { IsEmail, IsString, Matches, MinLength } from 'class-validator';
import { Transform } from 'class-transformer';

export class LoginDto {
  @ApiProperty({ example: 'user@example.com' })
  @Transform(({ value }) => (typeof value === 'string' ? value.trim().toLowerCase() : value))
  @IsEmail()
  email!: string;

  @ApiProperty({ example: 'Str0ngPass!' })
  @IsString()
  @MinLength(8)
  @Matches(/[A-Z]/, { message: 'password must contain an uppercase letter' })
  @Matches(/[0-9]/, { message: 'password must contain a digit' })
  password!: string;
}
