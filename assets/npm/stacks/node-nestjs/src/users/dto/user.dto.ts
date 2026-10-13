import { ApiProperty } from '@nestjs/swagger';

export class UserDto {
  @ApiProperty()
  id!: string;

  @ApiProperty()
  email!: string;

  @ApiProperty({ enum: ['USER', 'ADMIN'] })
  role!: string;

  @ApiProperty({ description: 'ISO-8601 timestamp' })
  createdAt!: string;
}
