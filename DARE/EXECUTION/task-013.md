# TASK 013: CI GHA + fixtures-inventory + fechamento ciclo 0

> **Complexidade:** MED  
> **Depends on:** task-003, task-012  
> **Estimativa:** 1 h

---

## 1. OBJETIVO

Ao final, o workflow `.github/workflows/governance-001.yml` está definido (Node 20 + verify-all + artifact do manifesto), `fixtures-inventory.md` lista as fixtures do Ciclo 0, e o fechamento do microplano 001 (exceto cargo/DEC-001) está documentado.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 5 + Fase 7 (N)
- **Decisões:** T-04 artefato manifesto; RF-12/RF-13 SHOULD

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `.github/workflows/governance-001.yml` | CI governança |
| MODIFICAR | `docs/compatibility/fixtures-inventory.md` | lista fixtures |
| MODIFICAR | `docs/compatibility/README.md` | release notes Ciclo 0 |
| MODIFICAR | `docs/DECISION-LOG.md` | nota épico Issues (RF-13) opcional |
| MODIFICAR | `DARE/TASKS.md` | marcar task-013 DONE + progresso |

---

## 4. IMPLEMENTAÇÃO

### Workflow mínimo

```yaml
name: governance-001
on:
  push:
    paths: ["docs/**", "scripts/governance/**", ".github/workflows/governance-001.yml"]
  pull_request:
    paths: ["docs/**", "scripts/governance/**", ".github/workflows/governance-001.yml"]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: "20" }
      - run: node scripts/governance/verify-all.mjs
      - uses: actions/upload-artifact@v4
        with:
          name: baseline-manifest
          path: docs/compatibility/baseline-manifest.json
```

### fixtures-inventory.md

Listar (nome + propósito 1 linha):  
`empty-project`, `existing-node-project`, `existing-rust-project`, `existing-python-project`, `monorepo`, `project-with-claude`, `project-with-cursor`, `project-with-codex`, `project-with-antigravity`, `project-with-all-harnesses`, `invalid-config`, `legacy-dag`, `customized-assets`, `windows-path-cases`.

### Fechamento

Seção README: critérios microplano 001 satisfeitos; cargo adiado DEC-001; próximo = `002-workspace-rust-e-toolchain.md`.

### Testes esperados

- [ ] YAML do workflow parseável (actionlint se disponível, senão revisão manual)
- [ ] `verify-all.mjs` exit 0 localmente
- [ ] Inventory contém ≥ 14 nomes de fixture
- [ ] Edge: paths filter do workflow inclui `docs/**` e `scripts/governance/**`

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Workflow não imprime secrets
- [ ] `actions/checkout` / `setup-node` / `upload-artifact` em majors pinadas (@v4)
- [ ] Artifact só manifesto (sem tarball privado)

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
node scripts/governance/verify-all.mjs
# Validar workflow existe e contém verify-all.mjs
rg -n "verify-all.mjs" .github/workflows/governance-001.yml
rg -n "windows-path-cases|legacy-dag|empty-project" docs/compatibility/fixtures-inventory.md
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Workflow não pode ser `run: echo ok` sem verify-all
- [ ] Inventory não pode ser lista vazia
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] CI definido + inventory + release notes
- [ ] Microplano 001 desbloqueia 002 (doc)
- [ ] `DARE/TASKS.md`: task-013 → DONE; progresso 13/13

---

## 9. PRÓXIMA TASK SUGERIDA

— (fim do DAG 001). Avançar para microplano 002 após aprovação humana.
