# TASK 005: Pacote compatibility + DECISION-LOG (DEC-001)

> **Complexidade:** MED  
> **Depends on:** task-001  
> **Estimativa:** 1,5 h

---

## 1. OBJETIVO

Ao final, `classification-matrix.md` lista CI-001..CI-014 classificados, as três políticas (idioma, disco/JSON, breaking) estão completas com regras fechadas, e `DECISION-LOG.md` contém DEC-001 (waiver cargo→002).

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 3
- **Decisões:** T-01 DEC-001; RF-07..RF-11; §5.6–5.7

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `docs/compatibility/classification-matrix.md` | CI-001..014 |
| MODIFICAR | `docs/compatibility/language-policy.md` | 4 regras |
| MODIFICAR | `docs/compatibility/disk-and-json-policy.md` | tabela §5.7 |
| MODIFICAR | `docs/compatibility/breaking-change-process.md` | máquina estados |
| MODIFICAR | `docs/DECISION-LOG.md` | DEC-001 |
| MODIFICAR | `docs/compatibility/README.md` | índice |

---

## 4. IMPLEMENTAÇÃO

### classification-matrix.md

Tabela com colunas: `item_id`, `class`, `summary`, `action`, `adr_ref`, `source`.  
Itens **obrigatórios** CI-001..CI-014 conforme BLUEPRINT §4.3 (copiar summaries/actions).

### language-policy.md

1. Docs governança: pt-BR  
2. Rust novo: en-US  
3. Strings PT Classe A: preservar até ADR-003  
4. Mistura no mesmo comando novo: proibida  

### disk-and-json-policy.md

Tabela completa §5.7 (9 linhas de política).

### breaking-change-process.md

Estados: Proposed → ADR Draft → Review(TL) → Review(PO) → Accepted → Changelog + Migration → Merge.  
Lista fechada de 5 breaking types. Pré-condição PR checklist.

### DEC-001

| decision_id | date | summary | adr_refs | owner | status |
|-------------|------|---------|----------|-------|--------|
| DEC-001 | 2026-07-20 | Gates cargo transferidos ao microplano 002 | n/a | Tech Lead DARE CLI | active |

### Testes esperados

- [ ] `grep CI-014` encontra linha class D
- [ ] `grep DEC-001` em DECISION-LOG
- [ ] Nenhum texto `não classificado` / `TBD class`
- [ ] Edge: item C sem adr_ref → inválido (CI-007..009 devem ter adr_ref)

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Classe D documentada como must_fix (RS-07)
- [ ] Sem exemplos com tokens reais

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
rg -n "CI-01[0-4]" docs/compatibility/classification-matrix.md
rg -n "DEC-001" docs/DECISION-LOG.md
rg -n "não classificado|TBD" docs/compatibility/ && exit 1 || exit 0
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Políticas não podem ser só “ver Blueprint”
- [ ] Matrix com menos de 14 itens = FAIL
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] 14 itens + DEC-001 + 3 políticas
- [ ] `DARE/TASKS.md`: task-005 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-012` (após 004 e 011)
