import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { ConfigService } from '@nestjs/config';
import * as bcrypt from 'bcrypt';
import { UsersRepository } from '../users/users.repository.js';
import { LoginDto } from './dto/login.dto.js';
import { LoginResponseDto } from './dto/login-response.dto.js';

@Injectable()
export class AuthService {
  constructor(
    private readonly users: UsersRepository,
    private readonly jwt: JwtService,
    private readonly config: ConfigService,
  ) {}

  async login(dto: LoginDto): Promise<LoginResponseDto> {
    const user = await this.users.findByEmail(dto.email);
    if (!user) throw new UnauthorizedException('Invalid credentials');
    const ok = await bcrypt.compare(dto.password, user.password);
    if (!ok) throw new UnauthorizedException('Invalid credentials');

    const payload = { sub: user.id, email: user.email, role: user.role };
    const accessToken = await this.jwt.signAsync(payload);
    const expiresIn = parseExpires(this.config.get<string>('JWT_EXPIRES_IN', '15m'));
    return { accessToken, expiresIn };
  }
}

function parseExpires(expr: string): number {
  const m = /^(\d+)([smhd])$/.exec(expr);
  if (!m) return 900;
  const n = Number(m[1]);
  const mult = { s: 1, m: 60, h: 3600, d: 86400 }[m[2] as 's' | 'm' | 'h' | 'd'];
  return n * mult;
}
