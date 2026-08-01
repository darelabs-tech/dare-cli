---
name: dare-laravel-api
description: Padrões DARE para APIs REST em Laravel 11 + PHP 8.3 — Strict Types, FormRequests, Services, JsonResources, Eloquent + casts, tratamento global de exceções, testes Feature/Pest, PHPStan/Larastan, Pint.
---

# DARE Laravel API Skill

Você é um desenvolvedor sênior PHP 8.3 / Laravel 11.x especializado em APIs REST. Seu objetivo é gerar código **limpo, legível, performático e fortemente tipado**, seguindo Layered Design e padrões DARE.

## Quando usar

- Projeto Laravel novo via DARE
- Adicionar feature em API Laravel existente
- Auditar projeto Laravel para conformidade DARE
- Migrar API antiga (Laravel 8/9/10) para padrões 11.x

## Stack canônica

- **PHP 8.3** — Strict Types em todos os arquivos (`declare(strict_types=1);`)
- **Laravel 11.x** — modo API (sem views server-side por default)
- **Banco** — PostgreSQL (preferido) ou MySQL 8
- **Testes** — Pest (preferido) ou PHPUnit 11
- **Análise estática** — PHPStan + Larastan (nível 8)
- **Formatação** — Laravel Pint (`pint`)
- **Auth** — Laravel Sanctum (tokens) ou Passport (OAuth)

## Layered Design em Laravel

Mapeamento DARE → Laravel:

| Camada DARE | Pasta Laravel |
|---|---|
| Handler | `app/Http/Controllers/` |
| Service | `app/Services/` |
| Repository | `app/Repositories/` |
| Model | `app/Models/` |
| Presenter | `app/Http/Resources/` |

### Controllers (Handlers)

- Apenas: receber request → chamar Service → retornar Resource
- NUNCA: lógica de negócio, queries Eloquent, validação inline

```php
<?php declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Requests\StoreUserRequest;
use App\Http\Resources\UserResource;
use App\Services\RegisterUser;

final class UserApiController extends Controller
{
    public function __construct(private RegisterUser $service) {}

    public function store(StoreUserRequest $request): UserResource
    {
        $user = $this->service->execute($request->validated());
        return new UserResource($user);
    }
}
```

### FormRequests (Validação)

NUNCA validar inline no Controller. Sempre FormRequest:

```php
<?php declare(strict_types=1);

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

final class StoreUserRequest extends FormRequest
{
    public function authorize(): bool
    {
        return $this->user()->can('create', User::class);
    }

    public function rules(): array
    {
        return [
            'email' => ['required', 'email', 'unique:users,email'],
            'password' => ['required', 'string', 'min:12'],
            'name' => ['required', 'string', 'max:255'],
        ];
    }
}
```

### Services (Business Logic)

Uma operação por classe. Strict types, exceções de domínio próprias.

```php
<?php declare(strict_types=1);

namespace App\Services;

use App\Exceptions\UserAlreadyExistsException;
use App\Models\User;
use App\Repositories\UserRepository;
use Illuminate\Support\Facades\Hash;

final class RegisterUser
{
    public function __construct(private UserRepository $users) {}

    public function execute(array $data): User
    {
        if ($this->users->existsByEmail($data['email'])) {
            throw new UserAlreadyExistsException();
        }
        return $this->users->create([
            'email' => $data['email'],
            'name' => $data['name'],
            'password' => Hash::make($data['password']),
        ]);
    }
}
```

### Repositories (Data Access)

Abstrai Eloquent. Service só conhece o Repository, não o Model direto.

```php
<?php declare(strict_types=1);

namespace App\Repositories;

use App\Models\User;

final class UserRepository
{
    public function existsByEmail(string $email): bool
    {
        return User::where('email', $email)->exists();
    }

    public function create(array $data): User
    {
        return User::create($data);
    }
}
```

### Resources (Presentation)

Nunca retorne Model direto do Controller. Sempre passe por JsonResource.

```php
<?php declare(strict_types=1);

namespace App\Http\Resources;

use Illuminate\Http\Resources\Json\JsonResource;

final class UserResource extends JsonResource
{
    public function toArray($request): array
    {
        return [
            'id' => $this->id,
            'email' => $this->email,
            'name' => $this->name,
            'createdAt' => $this->created_at?->toIso8601String(),
        ];
    }
}
```

### Models

- Definir `$fillable` ou `$guarded`
- `casts` corretos para tipos não-string
- Relacionamentos explícitos

```php
<?php declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

final class User extends Model
{
    protected $fillable = ['email', 'name', 'password'];
    protected $hidden = ['password'];
    protected $casts = [
        'email_verified_at' => 'datetime',
        'created_at' => 'datetime',
    ];
}
```

## Tratamento de exceções

Em `bootstrap/app.php` (Laravel 11):

```php
->withExceptions(function (Exceptions $exceptions) {
    $exceptions->render(function (UserAlreadyExistsException $e) {
        return response()->json(['error' => 'User already exists', 'code' => 'USER_EXISTS'], 409);
    });
    $exceptions->render(function (ValidationException $e) {
        return response()->json(['error' => 'Validation failed', 'errors' => $e->errors()], 422);
    });
})
```

## Transações de banco

Sempre que inserir/atualizar múltiplas tabelas:

```php
DB::transaction(function () use ($data) {
    $user = User::create($data);
    UserProfile::create(['user_id' => $user->id, ...]);
    AuditLog::log('user.created', $user);
});
```

## Testes (Pest)

Feature test para cada endpoint:

```php
<?php declare(strict_types=1);

use App\Models\User;

it('cria usuário com sucesso', function () {
    $payload = ['email' => 'jane@example.com', 'name' => 'Jane', 'password' => 'longsecret123'];

    $this->actingAs(User::factory()->create(['is_admin' => true]))
         ->postJson('/api/users', $payload)
         ->assertCreated()
         ->assertJsonStructure(['data' => ['id', 'email', 'name', 'createdAt']]);

    $this->assertDatabaseHas('users', ['email' => 'jane@example.com']);
});

it('rejeita email duplicado', function () {
    User::factory()->create(['email' => 'taken@example.com']);
    $this->actingAs(User::factory()->create(['is_admin' => true]))
         ->postJson('/api/users', ['email' => 'taken@example.com', 'name' => 'X', 'password' => 'longsecret123'])
         ->assertStatus(409);
});
```

Cobertura mínima:
- Resposta de sucesso (200/201)
- Erro de validação (422)
- Auth/Autz (401/403)
- Not Found (404)

## Padrões de código

- Type hints estritos em todos os métodos (`function x(int $a): User`)
- Evite `null` — prefira exceção tipada ou `Optional` (collection helper)
- PHPDoc apenas onde tipo nativo não dá conta (`/** @var User[] $users */`)
- Comentários explicam o **porquê**, não o **o quê**
- Use `readonly` em propriedades imutáveis (`public function __construct(public readonly UserRepository $repo) {}`)

## Validação estática

```bash
./vendor/bin/phpstan analyse --level=8
./vendor/bin/pint --test
./vendor/bin/pest --coverage --min=80
```

## Antipatterns

| AP | Antipattern | Sinal | Correção |
|---|---|---|---|
| AP-01 | Validação no Controller | `$request->validate([...])` | FormRequest dedicada |
| AP-02 | Query Eloquent no Controller | `User::where(...)->get()` | Repository |
| AP-03 | Lógica no Controller | `if ($x) { ... } else { ... }` longo | Service |
| AP-04 | Retornar Model direto | `return $user` | JsonResource |
| AP-05 | `$guarded = []` | mass assignment vulnerável | `$fillable` explícito |
| AP-06 | Sem `declare(strict_types=1)` | Type coercion silenciosa | Adicionar no topo |
| AP-07 | Múltiplas inserções sem transaction | Estado inconsistente em falha parcial | `DB::transaction()` |
| AP-08 | Senha hash com `bcrypt()` default cost | Custo baixo | `Hash::make($pwd, ['rounds' => 12])` |

## Segurança (combinar com dare-security)

- **Senhas:** `Hash::make()` com `rounds >= 12` ou Argon2
- **Auth:** Sanctum para SPA/mobile, OAuth (Passport) para 3rd-party
- **Rate limit:** `ThrottleRequests` middleware (`->middleware('throttle:60,1')`)
- **Headers:** middleware com HSTS, X-Frame-Options, CSP
- **CORS:** `config/cors.php` específico, nunca `*` em produção
- **Mass assignment:** `$fillable` SEMPRE, nunca `$guarded = []`

## Como aplicar

### Passo 1: Audit do projeto

```bash
grep -rn "request()->validate" app/Http/Controllers/   # AP-01
grep -rn "::where\|::find" app/Http/Controllers/        # AP-02
./vendor/bin/phpstan analyse --level=8                  # tipos
```

### Passo 2: Migrar Controllers para FormRequests

Para cada `$request->validate(...)`, gerar FormRequest correspondente:

```bash
php artisan make:request StoreUserRequest
```

### Passo 3: Extrair Services

Para cada Controller com lógica > 10 linhas, criar Service correspondente.

### Passo 4: Adicionar Repositories

Para cada Controller com query Eloquent direta, criar Repository.

### Passo 5: Adicionar Resources

Substituir `return $user` por `return new UserResource($user)`.

## Dicas

- **Combine** com `dare-docker` para containerizar (PHP-FPM + Nginx separados)
- **Use** `dare-security` para auditoria OWASP em FormRequests
- **Para realtime**, use Laravel Reverb (Pusher-compatible) ou Soketi

---

Esta skill é parte do DARE Method e está sob licença MIT.
