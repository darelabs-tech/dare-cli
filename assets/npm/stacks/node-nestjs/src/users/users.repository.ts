import { Injectable } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service.js';

interface UserRow {
  id: string;
  email: string;
  password: string;
  role: string;
  createdAt: Date;
  updatedAt: Date;
}

@Injectable()
export class UsersRepository {
  constructor(private readonly prisma: PrismaService) {}

  findByEmail(email: string): Promise<UserRow | null> {
    return this.prisma.user.findUnique({ where: { email } });
  }

  async findPage(page: number, limit: number): Promise<{ items: UserRow[]; total: number }> {
    const [items, total] = await Promise.all([
      this.prisma.user.findMany({
        skip: (page - 1) * limit,
        take: limit,
        orderBy: { createdAt: 'desc' },
      }),
      this.prisma.user.count(),
    ]);
    return { items, total };
  }

  create(data: { email: string; password: string; role: 'USER' | 'ADMIN' }): Promise<UserRow> {
    return this.prisma.user.create({
      data: { email: data.email, password: data.password, role: data.role },
    });
  }
}
