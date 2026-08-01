---
name: skill-nestjs-api
description: Padrões DARE para APIs REST em NestJS + TypeScript + Prisma + Swagger. Modules, Controllers, Services, DTOs com class-validator, Guards, Interceptors, exceções globais, Jest + Supertest, OpenAPI auto-gerado, rate limiting com @nestjs/throttler.
---

# DARE NestJS API Skill

Você é um desenvolvedor sênior TypeScript / NestJS especializado em APIs REST. Seu objetivo é gerar código **idiomático Nest, fortemente tipado, com auth/autz robustos e OpenAPI auto-gerado**, seguindo Layered Design DARE.

## Quando usar

- Projeto NestJS novo via DARE
- Adicionar feature em API NestJS existente
- Migrar API Express clássica para NestJS
- Auditar projeto NestJS para conformidade DARE

## Stack canônica

- **TypeScript 5.5+** com `strict: true`
- **NestJS 11.x** (módulos, providers, DI)
- **Prisma 5.x** ORM (PostgreSQL ou MySQL)
- **class-validator + class-transformer** para DTOs
- **@nestjs/swagger** para OpenAPI auto-gerado
- **@nestjs/throttler** para rate limiting
- **@nestjs/passport + @nestjs/jwt** para auth
- **Jest + Supertest** para testes
- **ESLint + Prettier** para formatação

## Layered Design em NestJS

Cada módulo segue:

```
src/<feature>/
├── <feature>.module.ts          ← define providers, controllers
├── <feature>.controller.ts      ← Handler
├── <feature>.service.ts         ← Service (uma operação) ou facade
├── <feature>.repository.ts      ← Repository (Prisma)
├── dto/
│   ├── create-<feature>.dto.ts  ← class-validator
│   └── update-<feature>.dto.ts
├── entities/
│   └── <feature>.entity.ts      ← Model
└── tests/
    ├── <feature>.service.spec.ts  ← unit
    └── <feature>.e2e-spec.ts      ← E2E
```

## Controllers (Handler)

```typescript
import { Body, Controller, Post, UseGuards } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiCreatedResponse } from '@nestjs/swagger';
import { JwtAuthGuard } from '@/auth/jwt-auth.guard';
import { CreateUserDto } from './dto/create-user.dto';
import { UserResponseDto } from './dto/user-response.dto';
import { RegisterUserService } from './register-user.service';

@ApiTags('users')
@Controller('users')
@UseGuards(JwtAuthGuard)
export class UsersController {
  constructor(private readonly register: RegisterUserService) {}

  @Post()
  @ApiOperation({ summary: 'Criar novo usuário' })
  @ApiCreatedResponse({ type: UserResponseDto })
  async create(@Body() dto: CreateUserDto): Promise<UserResponseDto> {
    const user = await this.register.execute(dto);
    return UserResponseDto.from(user);
  }
}
```

## DTOs (validação)

```typescript
import { IsEmail, IsString, MinLength } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';

export class CreateUserDto {
  @ApiProperty({ example: 'jane@example.com' })
  @IsEmail()
  email!: string;

  @ApiProperty({ example: 'Jane Doe' })
  @IsString()
  name!: string;

  @ApiProperty({ minLength: 12 })
  @IsString()
  @MinLength(12)
  password!: string;
}
```

`main.ts` ativa global pipe:
```typescript
app.useGlobalPipes(new ValidationPipe({ whitelist: true, forbidNonWhitelisted: true, transform: true }));
```

## Services (Business Logic)

Uma operação por classe:

```typescript
import { Injectable } from '@nestjs/common';
import { hash } from '@node-rs/argon2';
import { UsersRepository } from './users.repository';
import { CreateUserDto } from './dto/create-user.dto';
import { User } from './entities/user.entity';
import { UserAlreadyExistsError } from './errors';

@Injectable()
export class RegisterUserService {
  constructor(private readonly repo: UsersRepository) {}

  async execute(dto: CreateUserDto): Promise<User> {
    if (await this.repo.existsByEmail(dto.email)) {
      throw new UserAlreadyExistsError();
    }
    return this.repo.create({
      ...dto,
      password: await hash(dto.password),
    });
  }
}
```

## Repositories (Prisma)

```typescript
import { Injectable } from '@nestjs/common';
import { PrismaService } from '@/prisma/prisma.service';

@Injectable()
export class UsersRepository {
  constructor(private readonly prisma: PrismaService) {}

  async existsByEmail(email: string): Promise<boolean> {
    return !!(await this.prisma.user.findUnique({ where: { email } }));
  }

  async create(data: { email: string; name: string; password: string }) {
    return this.prisma.user.create({ data });
  }
}
```

## Tratamento global de exceções

```typescript
// src/common/filters/all-exceptions.filter.ts
@Catch()
export class AllExceptionsFilter implements ExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost) {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();

    if (exception instanceof UserAlreadyExistsError) {
      return response.status(409).json({ error: 'USER_EXISTS' });
    }
    if (exception instanceof HttpException) {
      return response.status(exception.getStatus()).json(exception.getResponse());
    }
    // log + 500 genérico
    return response.status(500).json({ error: 'INTERNAL' });
  }
}

// main.ts
app.useGlobalFilters(new AllExceptionsFilter());
```

## OpenAPI auto-gerado

```typescript
// main.ts
import { SwaggerModule, DocumentBuilder } from '@nestjs/swagger';

const config = new DocumentBuilder()
  .setTitle('Projeto API')
  .setVersion('1.0')
  .addBearerAuth()
  .build();

const document = SwaggerModule.createDocument(app, config);
SwaggerModule.setup('docs', app, document); // UI em /docs
writeFileSync('./public/openapi.json', JSON.stringify(document));
```

## Rate limiting

```typescript
import { ThrottlerModule } from '@nestjs/throttler';

@Module({
  imports: [
    ThrottlerModule.forRoot([{ ttl: 60_000, limit: 100 }]),
  ],
})

// Endpoint sensível
@Throttle({ default: { limit: 5, ttl: 900_000 } })  // 5/15min
@Post('login')
async login() {...}
```

## Auth (JWT + Passport)

```typescript
@Injectable()
export class JwtAuthGuard extends AuthGuard('jwt') {}

@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor(config: ConfigService) {
    super({
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      secretOrKey: config.getOrThrow('JWT_SECRET'),
    });
  }
  async validate(payload: any) {
    return { id: payload.sub, email: payload.email };
  }
}
```

## Testes

### Unit (service)

```typescript
describe('RegisterUserService', () => {
  it('falha se email já existe', async () => {
    const repo = { existsByEmail: jest.fn().mockResolvedValue(true) } as any;
    const sut = new RegisterUserService(repo);
    await expect(sut.execute({ email: 'x@y.com', name: 'X', password: 'longsecret123' }))
      .rejects.toThrow(UserAlreadyExistsError);
  });
});
```

### E2E (Supertest)

```typescript
describe('POST /users', () => {
  it('cria usuário com sucesso', async () => {
    const res = await request(app.getHttpServer())
      .post('/users')
      .set('Authorization', `Bearer ${adminToken}`)
      .send({ email: 'jane@example.com', name: 'Jane', password: 'longsecret123' });
    expect(res.status).toBe(201);
    expect(res.body.email).toBe('jane@example.com');
  });
});
```

## Antipatterns

| AP | Antipattern | Correção |
|---|---|---|
| AP-01 | Validação inline no Controller | DTO + ValidationPipe |
| AP-02 | Prisma direto no Controller | Repository |
| AP-03 | Lógica no Controller | Service |
| AP-04 | Sem DTO de saída | `UserResponseDto.from(user)` |
| AP-05 | `JwtStrategy` com secret hardcoded | `ConfigService.getOrThrow` |
| AP-06 | Sem `@nestjs/throttler` em login | rate limit obrigatório |
| AP-07 | OpenAPI escrito à mão | `@nestjs/swagger` decorators |
| AP-08 | Erros sem filter global | inconsistência de response |

## Segurança (combinar com `dare-security`)

- Hash com `@node-rs/argon2` (Argon2id)
- JWT: RS256 para tokens públicos, HS256 + secret ≥256 bits para internos
- `helmet` middleware para headers
- CORS específico, nunca `*`
- Rate limit em login (5/15min) + APIs públicas
- Refresh token com rotação no DB

## Validação no CI

```bash
npm run lint
npm run test
npm run test:e2e
npm run build
npx prisma migrate deploy --dry-run     # valida migrations
npm audit --audit-level=high
```

## Como aplicar

### Passo 1: Audit

```bash
grep -rn "@Body()" src/ | grep -v "Dto"        # AP-01
grep -rn "prisma\." src/*/controllers/         # AP-02
grep -rn "JwtStrategy" src/ | grep -v Config   # AP-05
```

### Passo 2: Migrar para DTOs

Para cada `@Body() body: any`, criar DTO com class-validator + `@ApiProperty`.

### Passo 3: Extrair Services

Toda lógica > 5 linhas em Controller → Service injetável.

### Passo 4: Adicionar Repositories

Encapsular Prisma. Controller chama Service, Service chama Repository.

### Passo 5: Configurar Swagger + throttler

`main.ts` com `SwaggerModule.setup` e exportar `public/openapi.json`. Adicionar `ThrottlerModule` global.

## Dicas

- **Combine** com `dare-docker` para PHP-FPM-style separação (Node + Postgres + Redis)
- **Use** `dare-llm-integration` se houver chamadas LLM
- **Para realtime**, use `@nestjs/websockets` + skill `dare-realtime`

---

Esta skill é parte do DARE Method e está sob licença MIT.
