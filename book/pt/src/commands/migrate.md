# `dare migrate`

Gera a estratégia de migração e o plano de transição técnica a partir de um projeto legado para uma nova tecnologia-alvo, produzindo cenários Gherkin (BDD) de paridade funcional para testes e validação.

## Uso

```bash
dare migrate --to <STACK> [OPTIONS]
```

## Flags

| Flag | Tipo | Obrigatório | Descrição |
|---|---|---|---|
| `--to <STACK>` | string | Sim | Stack técnica de destino (veja a allowlist abaixo) |
| `--dir <PATH>` | path | Não | Diretório raiz do projeto alternativo (padrão: cwd) |
| `--check` | bool | Não | Apenas analisa e exibe o relatório de migração no terminal (não altera arquivos no disco) |
| `--json` | bool | Não | Saída estruturada formatada em JSON |

---

## Pré-requisitos para Execução

Para rodar o `dare migrate`, a CLI exige o estado da engenharia reversa do legado:
1. Existência do arquivo de visão de negócio `DARE/IDEIA.md`.
2. Presença do arquivo de fatos `DARE/REVERSE/reverse-facts.json` (ou arquivos `module-*.md` na pasta `REVERSE/`).
3. Se os artefatos `DARE/PROJECT-DNA.md` ou `DARE/PATTERNS.md` estiverem presentes no projeto, eles serão consumidos para refinar a estratégia de migração; caso contrário, a CLI emite avisos (`warnings`).

---

## Allowlist `--to`

O valor da flag `--to` é case-sensitive e deve pertencer à lista de stacks válidas do DARE:

- `node-nestjs`
- `python-fastapi`
- `php-laravel`
- `go-gin`
- `go-stdlib`
- `rails`
- `rust-axum`
- `rust` (mapeado internamente como a família `rust`, persistido como literal)
- `rust-leptos`
- `rust-leptos-csr`
- `react`
- `vue`
- `mcp-node-ts`

---

## Estrutura da Migração Gerada

Quando executado, o comando cria a estrutura sob `DARE/MIGRATION/`:

### 1. `MIGRATION.md`
Plano técnico dividido em 3 fases canônicas estruturadas:
- **Fase 1: Foundations** (Configuração do novo projeto, setup da stack-alvo, banco de dados e middlewares).
- **Fase 2: Modules** (Migração módulo a módulo baseando-se no mapeamento da engenharia reversa).
- **Fase 3: Cutover** (Estratégia de implantação, sincronização de dados e rollback).

### 2. `parity/*.feature`
Cenários BDD escritos em formato Gherkin para cada módulo detectado do legado. Esses arquivos de teste de aceitação garantem que a nova implementação atenda exatamente ao mesmo comportamento funcional do legado.

---

## Exemplos de Uso

```bash
# Planeja e gera os artefatos de migração do legado para Rust Axum
dare migrate --to rust-axum

# Apenas diagnostica a estratégia sem criar arquivos no disco
dare migrate --to python-fastapi --check
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Migração planejada e gerada com sucesso (ou modo `--check` finalizado) |
| `1` | Erro interno inesperado após validação de parâmetros |
| `2` | Uso inválido de argumentos (ex.: ausência da flag obrigatória `--to`) |
| `3` | O diretório especificado em `--dir` não foi encontrado |
| `4` | Entrada inválida (como stack de destino fora da allowlist, falta do arquivo `IDEIA.md`/módulos) |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
