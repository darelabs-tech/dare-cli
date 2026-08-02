# `dare init`

Inicializa a infraestrutura DARE em um projeto **novo** (greenfield).

## Uso

```bash
dare init [OPTIONS]
```

## Flags

| Flag | Tipo | Descrição |
|---|---|---|
| `--stack <STACK>` | string | Stack do projeto (ex.: `rust`, `python`, `node`) |
| `--mcp <LANG>` | string | Inicializa como MCP server (ex.: `rust`, `python`, `ts`) |
| `--fullstack` | bool | Inclui scaffold frontend (`react` ou `vue`) |
| `--non-interactive` | bool | Modo não-interativo (requer `--stack` ou `--mcp`) |
| `--force` | bool | Sobrescreve diretório existente |
| `--check` | bool | Verifica sem escrever nada (dry-run) |
| `--dir <PATH>` | path | Diretório destino (padrão: `.`) |

## Stacks disponíveis

| Alias CLI | Stack ID | Descrição |
|---|---|---|
| `rust` | `rust` | Rust (Cargo workspace) |
| `python` | `python` | Python + FastAPI |
| `node`, `ts` | `node-ts` | Node.js + TypeScript |
| `go` | `go` | Go + Gin |
| `laravel`, `php` | `php-laravel-11` | PHP + Laravel 11 |
| `rails` | `ruby-rails-8` | Ruby on Rails 8 |
| `nest` | `nestjs` | NestJS + TypeScript |

### MCP servers

| Input `--mcp` | Stack ID |
|---|---|
| `ts`, `node`, `typescript` | `mcp-node-ts` |
| `python`, `py` | `mcp-python` |
| `rust` | `mcp-rust` |
| `go` | `mcp-go` |

## Exemplos

```bash
# Interativo (recomendado para projetos novos)
dare init

# Não-interativo com stack Rust
dare init --stack rust --non-interactive

# Projeto fullstack com frontend React
dare init --stack laravel --fullstack

# MCP server em Python
dare init --mcp python --non-interactive

# Dry-run (verifica sem escrever)
dare init --stack rust --check
```

## O que é criado?

Após `dare init`, a estrutura mínima gerada é:

```
./
├── dare.config.json          ← configuração do projeto
├── DARE/                     ← diretório de artefatos
└── .agents/
    ├── AGENTS.md             ← contexto para agentes de IA
    └── skills/               ← skills DARE para cada IDE
        ├── dare-design/
        ├── dare-blueprint/
        └── ...
```

Harnesses instalados automaticamente para os 4 agentes: **Antigravity**, **Claude Code**, **Cursor**, **Codex**.

## Exit codes

| Código | Quando |
|---|---|
| `0` | Sucesso |
| `2` | Uso inválido (`--stack` e `--mcp` juntos, falta de `--stack` em non-interactive) |
| `4` | Input inválido (nome do projeto, diretório já existe sem `--force`) |
| `5` | Erro de I/O |

## Próximo passo

```bash
dare bootstrap   # materializa o scaffold da stack
```
