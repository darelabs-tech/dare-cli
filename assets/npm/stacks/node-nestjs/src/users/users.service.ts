import { ConflictException, Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as bcrypt from 'bcrypt';
import { UsersRepository } from './users.repository.js';
import { UserDto } from './dto/user.dto.js';
import { CreateUserDto } from './dto/create-user.dto.js';

@Injectable()
export class UsersService {
  constructor(
    private readonly repo: UsersRepository,
    private readonly config: ConfigService,
  ) {}

  async list(page = 1, limit = 20): Promise<{ items: UserDto[]; total: number; page: number }> {
    const safePage = Math.max(1, page);
    const safeLimit = Math.min(100, Math.max(1, limit));
    const { items, total } = await this.repo.findPage(safePage, safeLimit);
    return { items: items.map(toDto), total, page: safePage };
  }

  async create(input: CreateUserDto): Promise<UserDto> {
    const existing = await this.repo.findByEmail(input.email);
    if (existing) throw new ConflictException('email already in use');
    const cost = Number(this.config.get<string>('BCRYPT_COST', '12'));
    const hash = await bcrypt.hash(input.password, cost);
    const created = await this.repo.create({
      email: input.email,
      password: hash,
      role: input.role ?? 'USER',
    });
    return toDto(created);
  }
}

function toDto(u: { id: string; email: string; role: string; createdAt: Date }): UserDto {
  return { id: u.id, email: u.email, role: u.role, createdAt: u.createdAt.toISOString() };
}
