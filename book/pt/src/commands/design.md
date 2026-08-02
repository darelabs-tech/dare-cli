# `dare design`

Gera o `DARE/DESIGN.md` — o documento de requisitos que guia todo o ciclo de desenvolvimento.

## Uso

```bash
dare design [DESCRIÇÃO] [OPTIONS]
dare design --interactive
```

## Flags

| Flag | Tipo | Descrição |
|---|---|---|
| `--interactive` | bool | Questionário guiado via TTY |
| `--dir <PATH>` | path | Diretório raiz do projeto |
| `--json` | bool | Saída em JSON |

## Exemplos

```bash
# Descrição direta
dare design "API REST de autenticação JWT com refresh token em Rust"

# Modo interativo (recomendado)
dare design --interactive

# Com saída JSON (para agentes)
dare design "..." --json
```

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "status": "ok",
  "design_path": "DARE/DESIGN.md",
  "sections_written": ["description", "objectives", "functional-requirements"],
  "enrichable_sections": ["description", "objectives", "functional-requirements", "stack"]
}
```

## Estrutura do DESIGN.md gerado

```markdown
# DESIGN: [Nome do projeto]

## 1. DESCRIÇÃO
<!-- AGENT:BEGIN section="description" -->
...
<!-- AGENT:END section="description" -->

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO
<!-- AGENT:BEGIN section="objectives" -->
...
<!-- AGENT:END section="objectives" -->

## 3. STAKEHOLDERS
...

## 4. REQUISITOS FUNCIONAIS
<!-- AGENT:BEGIN section="functional-requirements" -->
| ID | Requisito | Prioridade | Critério de aceite |
...
<!-- AGENT:END section="functional-requirements" -->

## 5. REQUISITOS NÃO-FUNCIONAIS
...
```

Os marcadores `<!-- AGENT:BEGIN/END -->` delimitam seções enriquecíveis por `dare ai`.

## Limites

| Item | Limite |
|---|---|
| Tamanho da descrição | 32.768 bytes UTF-8 |
| Leitura de DESIGN.md existente | 262.144 bytes |

## Exit codes

| Código | Quando |
|---|---|
| `0` | Sucesso |
| `2` | `--interactive` sem TTY |
| `4` | Descrição vazia, oversize, path fora do projeto |
| `5` | Erro de I/O |

## Próximo passo

```bash
dare blueprint   # gera arquitetura a partir do DESIGN.md
```
