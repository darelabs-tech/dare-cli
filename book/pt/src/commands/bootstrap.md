# `dare bootstrap`

Materializa o scaffold da stack no projeto atual, após `dare init`.

## Uso

```bash
dare bootstrap [OPTIONS]
```

## O que faz

- Cria a estrutura de diretórios da stack (ex.: `src/`, `tests/`, `Cargo.toml` para Rust)
- Aplica templates de configuração (ex.: `Cargo.toml`, `pyproject.toml`, `package.json`)
- Instala harnesses para os 4 agentes de IA (Antigravity, Claude, Cursor, Codex)
- Configura toolchain da stack

## Flags

| Flag | Descrição |
|---|---|
| `--check` | Verifica sem escrever nada (dry-run para CI) |
| `--json` | Saída em JSON |

## Comportamento de conflito

Se arquivos já existem, o `dare bootstrap` usa `SkipExisting` — cria o que falta, pula o que já existe. Não sobrescreve sem `--force`.

```bash
dare bootstrap          # SkipExisting (idempotente)
dare bootstrap --force  # sobrescreve tudo
```

## Próximo passo

```bash
dare design "descrição do projeto"
```
