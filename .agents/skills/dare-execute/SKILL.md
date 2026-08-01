---
name: dare-execute
description: Executa uma task específica com implementação de código e testes. Use quando o usuário aprovar TASKS.md e quiser executar uma task. Implementa o código, roda testes (Ralph Loop) e valida até passar.
---

# DARE Execute Skill

Você é um desenvolvedor especializado em implementação de código de alta qualidade. Seu objetivo é executar uma task específica, implementar o código conforme especificação e validar com testes.

## Quando usar esta skill

- TASKS.md foi aprovado pelo usuário
- Usuário quer executar uma task específica
- Precisa-se implementar código e testes
- Quarta fase do Método DARE (Execução)

## Como usar

### Passo 1: Ler a Especificação da Task
Leia o arquivo `DARE/EXECUTION/task-[id].md` que será executada. Extraia:
- Objetivo da task
- Arquivos a criar/modificar
- Validações (Validation Gates)
- Testes esperados
- Segurança

### Passo 2: Analisar Contexto
Leia os arquivos de contexto:
- `.agents/rules/dare-workflow.md`
- Exemplos em `examples/`
- Código existente no projeto

### Passo 3: Implementar o Código
Crie/modifique os arquivos conforme especificação:
- Siga os padrões do projeto
- Implemente validações
- Adicione comentários
- Mantenha código limpo

### Passo 4: Escrever Testes
Para cada arquivo criado, crie testes:
- Testes unitários
- Testes de integração
- Testes de segurança
- Validation Gates

### Passo 5: Ralph Loop (Validação Automática)
Execute os testes:

```bash
# Exemplo para Laravel
php artisan test tests/Feature/AuthTest.php
```

**Se os testes falharem:**
1. Leia o erro
2. Corrija o código
3. Rode os testes novamente
4. Repita até passar

**Se os testes passarem:**
1. Valide Validation Gates
2. Revise o código
3. Confirme com o usuário

### Passo 6: Criar Artifact de Progresso
Crie um Task Group Artifact mostrando:
- Task completada
- Arquivos criados
- Testes passando
- Próxima task

## Ralph Loop Detalhado

O Ralph Loop é um processo de validação automática:

```
1. Implementar código
   ↓
2. Escrever testes
   ↓
3. Rodar testes
   ↓
4. Testes passam? ✓ → Próxima task
                 ✗ → Ler erro
                     ↓
                     Corrigir código
                     ↓
                     Rodar testes (volta ao passo 3)
```

## Exemplo: Task 001 - Criar Migrations

### Passo 1: Ler Especificação
```
# Task 001: Criar Migrations de Users

Objetivo: Criar migrations para users e refresh_tokens

Arquivos a Criar:
- database/migrations/YYYY_MM_DD_HHMMSS_create_users_table.php
- database/migrations/YYYY_MM_DD_HHMMSS_create_refresh_tokens_table.php

Validações:
- [ ] Migrations criadas sem erros
- [ ] Tabelas têm índices apropriados
- [ ] Foreign keys estão corretas
- [ ] `php artisan migrate` executa sem erros
```

### Passo 2: Implementar Migrations

**Arquivo: database/migrations/2026_04_14_100000_create_users_table.php**
```php
<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('users', function (Blueprint $table) {
            $table->uuid('id')->primary();
            $table->string('email')->unique();
            $table->string('password_hash');
            $table->string('name');
            $table->boolean('is_active')->default(true);
            $table->timestamps();
            
            // Índices
            $table->index('email');
            $table->index('created_at');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('users');
    }
};
```

### Passo 3: Escrever Testes

**Arquivo: tests/Feature/MigrationTest.php**
```php
<?php

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class MigrationTest extends TestCase
{
    use RefreshDatabase;

    public function test_users_table_created()
    {
        $this->assertTrue(
            \Schema::hasTable('users')
        );
    }

    public function test_users_table_has_required_columns()
    {
        $this->assertTrue(
            \Schema::hasColumns('users', [
                'id', 'email', 'password_hash', 'name', 
                'is_active', 'created_at', 'updated_at'
            ])
        );
    }

    public function test_email_is_unique()
    {
        $this->assertTrue(
            \Schema::hasColumn('users', 'email')
        );
    }
}
```

### Passo 4: Rodar Testes

```bash
php artisan migrate
php artisan test tests/Feature/MigrationTest.php
```

**Saída esperada:**
```
✓ test_users_table_created
✓ test_users_table_has_required_columns
✓ test_email_is_unique

Tests: 3 passed
```

### Passo 5: Validar Validation Gates

```bash
# ✓ Migrations criadas sem erros
php artisan migrate

# ✓ Tabelas têm índices apropriados
php artisan tinker
>>> Schema::getColumnListing('users')

# ✓ Foreign keys estão corretas
# (Verificado no código)

# ✓ php artisan migrate executa sem erros
php artisan migrate:rollback
php artisan migrate
```

## Boas Práticas

1. **Siga Padrões:** Use convenções do projeto
2. **Teste Tudo:** Cobertura de testes alta
3. **Segurança:** Implemente proteções OWASP
4. **Documentação:** Adicione comentários
5. **Limpo:** Código legível e manutenível

## Segurança em Execução

Para cada task, verifique:
- Validação de entrada
- Autenticação/Autorização
- Criptografia de dados sensíveis
- Proteção contra SQL Injection
- Proteção contra XSS
- Rate Limiting (se aplicável)

## Dicas para Melhor Resultado

- **Contexto:** Leia exemplos em `examples/`
- **Padrões:** Siga convenções do projeto
- **Testes:** Use `skill-security` para testes de segurança
- **Feedback:** Peça aprovação após cada task
- **Ralph Loop:** Não pule validações

## Próxima Task

Após completar uma task:
1. Crie Artifact de progresso
2. Peça aprovação do usuário
3. Execute a próxima task
4. Repita até completar todas

## Exemplo de Artifact de Progresso

```
✓ Task 001: Criar Migrations de Users
  - Migrations criadas: 2
  - Testes passando: 3/3
  - Validation Gates: 4/4 ✓
  
Próxima: Task 002: Configurar JWT
```
