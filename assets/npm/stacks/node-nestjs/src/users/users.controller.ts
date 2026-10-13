import {
  Body,
  Controller,
  ForbiddenException,
  Get,
  Post,
  Query,
  Req,
  UseGuards,
} from '@nestjs/common';
import { AuthGuard } from '@nestjs/passport';
import { ApiBearerAuth, ApiTags } from '@nestjs/swagger';
import type { Request } from 'express';
import { UsersService } from './users.service.js';
import { UserDto } from './dto/user.dto.js';
import { CreateUserDto } from './dto/create-user.dto.js';

@ApiTags('users')
@ApiBearerAuth()
@UseGuards(AuthGuard('jwt'))
@Controller('users')
export class UsersController {
  constructor(private readonly users: UsersService) {}

  @Get()
  list(
    @Query('page') page = '1',
    @Query('limit') limit = '20',
  ): Promise<{ items: UserDto[]; total: number; page: number }> {
    return this.users.list(Number(page), Number(limit));
  }

  @Post()
  create(@Req() req: Request, @Body() dto: CreateUserDto): Promise<UserDto> {
    const caller = req.user as { role: string } | undefined;
    if (!caller || caller.role !== 'ADMIN') {
      throw new ForbiddenException('admin role required');
    }
    return this.users.create(dto);
  }
}
