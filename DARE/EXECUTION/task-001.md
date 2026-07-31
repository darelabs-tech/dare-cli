# TASK 001: Scaffold árvore docs/ (placeholders obrigatórios)

> **Complexidade:** LOW  
> **Depends on:** —  
> **Estimativa:** 0,5 h

---

## 1. OBJETIVO

Ao final, todos os paths obrigatórios de `docs/adr`, `docs/compatibility` e `docs/DECISION-LOG.md` existem com placeholders UTF-8 válidos (ADRs em `Proposed`, manifesto com hash placeholder).

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 1 — Containerização e setup
- **Arquivos existentes:** `DARE/BLUEPRINT.md` §3, §4.1–4.2
- **Decisões:** T-01 (sem Cargo); docs em pt-BR; schema manifesto `1.0`

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `docs/adr/README.md` | Índice stub |
| CRIAR | `docs/adr/ADR-001-compatibilidade-bugs-legados.md` | Frontmatter Proposed + 5 headings |
| CRIAR | `docs/adr/ADR-002-contrato-saida-json.md` | idem |
| CRIAR | `docs/adr/ADR-004-rest-compativel-e-mcp-real.md` | idem |
| CRIAR | `docs/adr/ADR-006-compatibilidade-migracao-graph-db.md` | idem |
| CRIAR | `docs/adr/ADR-007-formato-canonico-capabilities.md` | idem |
| CRIAR | `docs/compatibility/README.md` | Índice stub |
| CRIAR | `docs/compatibility/baseline-3.18.1.md` | Placeholder narrativo |
| CRIAR | `docs/compatibility/baseline-manifest.json` | Schema §4.1 com hash `a`×64 |
| CRIAR | `docs/compatibility/classification-matrix.md` | Stub |
| CRIAR | `docs/compatibility/language-policy.md` | Stub |
| CRIAR | `docs/compatibility/disk-and-json-policy.md` | Stub |
| CRIAR | `docs/compatibility/breaking-change-process.md` | Stub |
| CRIAR | `docs/compatibility/fixtures-inventory.md` | Stub |
| CRIAR | `docs/DECISION-LOG.md` | Stub tabela |

---

## 4. IMPLEMENTAÇÃO

### Passo 1: Criar diretórios `docs/adr` e `docs/compatibility`

### Passo 2: ADRs com frontmatter

```yaml
---
id: ADR-001
title: "Compatibilidade de bugs legados"
status: Proposed
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance"]
---
```

Headings exatos nesta ordem: `## Contexto`, `## Decisão`, `## Consequências`, `## Critérios de aceite`, `## Referências` — corpo de 1 linha cada.

### Passo 3: `baseline-manifest.json`

Campos obrigatórios BLUEPRINT §4.1; `content_hash` = 64 caracteres `a`; `resolved_url` = `https://registry.npmjs.org/@dewtech/dare-cli/-/dare-cli-3.18.1.tgz`.

### Testes esperados

- [ ] `test_all_required_paths_exist` — lista §3 completa presente
- [ ] `test_manifest_parses_as_json` — JSON.parse OK
- [ ] `test_adr_frontmatter_has_status_proposed` — grep `status: Proposed` nos 5 ADRs

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Sem secrets no manifesto/notes
- [ ] Sem PII (usar `dare-labs` como recorded_by)
- [ ] Não criar `.env` com valores

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
# Verificar paths (PowerShell)
$paths = @(
  "docs/adr/README.md",
  "docs/adr/ADR-001-compatibilidade-bugs-legados.md",
  "docs/adr/ADR-002-contrato-saida-json.md",
  "docs/adr/ADR-004-rest-compativel-e-mcp-real.md",
  "docs/adr/ADR-006-compatibilidade-migracao-graph-db.md",
  "docs/adr/ADR-007-formato-canonico-capabilities.md",
  "docs/compatibility/README.md",
  "docs/compatibility/baseline-3.18.1.md",
  "docs/compatibility/baseline-manifest.json",
  "docs/compatibility/classification-matrix.md",
  "docs/compatibility/language-policy.md",
  "docs/compatibility/disk-and-json-policy.md",
  "docs/compatibility/breaking-change-process.md",
  "docs/compatibility/fixtures-inventory.md",
  "docs/DECISION-LOG.md"
)
$paths | ForEach-Object { if (-not (Test-Path $_)) { throw "missing $_" } }
Get-Content docs/compatibility/baseline-manifest.json | ConvertFrom-Json | Out-Null
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

Placeholders de **conteúdo** de ADR são permitidos nesta task (status Proposed); proibido deixar arquivo vazio (0 bytes) ou sem frontmatter/headings.

- [ ] Nenhum `TODO`/`FIXME` em comentários de código (N/A se só markdown)
- [ ] Manifesto não é `{}` vazio
- [ ] `dare review task-001` passa se disponível

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Gates passaram
- [ ] 15 paths criados
- [ ] Sem `Cargo.toml` criado
- [ ] `DARE/TASKS.md`: task-001 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-002` (paralela no rank 0) ou após ambas: `task-003`, `task-004`, `task-005`…
