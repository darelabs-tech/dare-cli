import { ApiProperty } from '@nestjs/swagger';
import { IsEmail, IsEnum, IsOptional, IsString, Matches, MinLength } from 'class-validator';
import { Transform } from 'class-transformer';

export class CreateUserDto {
  @ApiProperty()
  @Transform(({ value }) => (typeof value === 'string' ? value.trim().toLowerCase() : value))
  @IsEmail()
  email!: string;

  @ApiProperty()
  @IsString()
  @MinLength(8)
  @Matches(/[A-Z]/)
  @Matches(/[0-9]/)
  password!: string;

  @ApiProperty({ enum: ['USER', 'ADMIN'], required: false, default: 'USER' })
  @IsOptional()
  @IsEnum(['USER', 'ADMIN'])
  role?: 'USER' | 'ADMIN';
}
