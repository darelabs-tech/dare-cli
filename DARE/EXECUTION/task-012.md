# TASK 012: Auditoria segurança npm + NO_SECRETS

> **Complexidade:** HIGH  
> **Depends on:** task-004, task-005, task-011  
> **Estimativa:** 1 h

---

## 1. OBJETIVO

Ao final, `npm audit --audit-level=high` em `scripts/governance` passa sem HIGH/CRITICAL, `verify-all.mjs` exit 0 inclui NO_SECRETS, e `docs/compatibility/README.md` documenta o Security gate 001 (RS-01..RS-07).

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 6 (N-1 auditoria)
- **Decisões:** DEC-001 — sem cargo; RS-02/04/05

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `scripts/governance/package.json` | deps pinadas se houver |
| MODIFICAR | `scripts/governance/verify-all.mjs` | garantir NO_SECRETS |
| MODIFICAR | `docs/compatibility/README.md` | seção Security gate 001 |
| MODIFICAR | `.env.governance.example` | confirmar só nomes de vars |

---

## 4. IMPLEMENTAÇÃO

### Passo 1: Audit

```bash
cd scripts/governance && npm audit --audit-level=high
```

Se zero deps: audit ainda deve “passar” (documentar). Se adicionar deps: pin + re-audit.

### Passo 2: NO_SECRETS

Função compartilhada ou reuso: varrer manifesto + ADRs; falhar verify-all se match.

### Passo 3: README Security gate 001

Tabela RS-01..RS-07 → artefato (ADR-001, matrix, scripts, DEC-001, etc.).

### Testes esperados

- [ ] `npm audit` exit 0 (ou sem vulnerabilidades HIGH+)
- [ ] `verify-all.mjs` exit 0
- [ ] Injetar `ghp_test` temporário num ADR de fixture → verify falha NO_SECRETS
- [ ] Edge: `.env.governance.example` sem valores secretos

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] RS-04 audit
- [ ] RS-02/05 sem secrets em docs
- [ ] Não logar tokens em erros de download

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
cd scripts/governance && npm audit --audit-level=high
node scripts/governance/verify-all.mjs
dare review task-012  # se CLI disponível
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Não “passar” audit ignorando HIGH
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Audit limpo + verify-all 0 + seção Security
- [ ] `DARE/TASKS.md`: task-012 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-013`
