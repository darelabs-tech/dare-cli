# TASK 011: verify-adr-frontmatter + índices README

> **Complexidade:** MED  
> **Depends on:** task-002, task-006, task-007, task-008, task-009, task-010  
> **Estimativa:** 1,5 h

---

## 1. OBJETIVO

Ao final, `verify-adr-frontmatter.mjs` exit 0 nas 5 ADRs Accepted, `verify-all.mjs` orquestra structure+adr(+baseline), e `docs/adr/README.md` lista as 5 com status Accepted.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 4 fechamento · §5.2
- **Pré-condição:** ADRs 001/002/004/006/007 já Accepted

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `scripts/governance/verify-adr-frontmatter.mjs` | Regras §5.2 |
| CRIAR | `scripts/governance/verify-adr-frontmatter.test.mjs` | Testes |
| CRIAR | `scripts/governance/fixtures/adr-valid.md` | Accepted completo |
| CRIAR | `scripts/governance/fixtures/adr-proposed.md` | Proposed (deve falhar regra) |
| MODIFICAR | `scripts/governance/verify-all.mjs` | structure + adr + baseline |
| MODIFICAR | `docs/adr/README.md` | Índice com links + Accepted |

---

## 4. IMPLEMENTAÇÃO

### Regras (rule ids estáveis)

| rule | Validação |
|------|-----------|
| `ADR_FILE_REQUIRED` | Ficheiros ADR-001,002,004,006,007 existem |
| `FRONTMATTER_PRESENT` | Bloco `---` no topo |
| `STATUS_ACCEPTED` | `status: Accepted` |
| `ID_MATCH_FILENAME` | id == prefixo ficheiro |
| `SECTIONS_ORDER` | 5 headings na ordem |
| `NO_SECRETS` | scan token=/Bearer /npm_/ghp_/AKIA |

ADR-003 presente → **ignorado** (não falha). Exit 0 se `errors.length===0` else 1.

### verify-all.mjs

Ordem: structure → adr-frontmatter → baseline (se ficheiro existir). Exit = máximo dos códigos (baseline pode ser 2).

### Testes esperados

- [ ] `should_fail_STATUS_ACCEPTED_on_proposed_fixture`
- [ ] `should_pass_on_repo_adrs_when_accepted`
- [ ] `should_not_fail_if_adr_003_extra_exists`
- [ ] Edge: ficheiro ADR-001 em falta → `ADR_FILE_REQUIRED`

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] NO_SECRETS aplicado a ADRs
- [ ] Sem shell concat ao listar ficheiros (`fs.readdir` + path)

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
node --test scripts/governance/verify-adr-frontmatter.test.mjs
node scripts/governance/verify-adr-frontmatter.mjs
node scripts/governance/verify-all.mjs
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Não hardcodar `ok:true` sem ler ficheiros
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Exit 0 nos verifies
- [ ] README com 5 links Accepted
- [ ] `DARE/TASKS.md`: task-011 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-012`
