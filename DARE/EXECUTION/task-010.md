# TASK 010: ADR-007 Formato canônico de capabilities

> **Complexidade:** MED  
> **Depends on:** task-001  
> **Estimativa:** 0,75 h

---

## 1. OBJETIVO

Ao final, ADR-007 Accepted distingue skills-pacote vs capabilities IDE, lista campos canônicos e os quatro adapters de harness — sem criar a matrix YAML (microplano 010).

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 4 · §5.5 ADR-007

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `docs/adr/ADR-007-formato-canonico-capabilities.md` | Conteúdo Accepted |

---

## 4. IMPLEMENTAÇÃO

### Decisão obrigatória

- Skills-pacote (`dare skill`) ≠ capabilities de IDE
- Campos: `id`, `title`, `description`, `instructions`, `cli_commands`, `outputs`, `assets`
- Referência futura: `assets/capability-matrix.yml` (criar no microplano 010 — não criar agora)
- Adapters: Claude, Cursor, Codex, Antigravity

Frontmatter Accepted + 5 headings.

### Testes esperados

- [ ] Distingue skill vs capability
- [ ] Lista os 7 campos canônicos
- [ ] Cita os 4 harnesses
- [ ] `status: Accepted`
- [ ] Edge: **não** cria `assets/capability-matrix.yml`

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Capabilities não embutem secrets em exemplos de instructions

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
rg -n "^status: Accepted" docs/adr/ADR-007-formato-canonico-capabilities.md
rg -n "cli_commands|Claude|Cursor|Codex|Antigravity" docs/adr/ADR-007-formato-canonico-capabilities.md
Test-Path assets/capability-matrix.yml; if ($?) { throw "matrix não deve existir ainda" }
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Sem Decisão TBD
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Contrato travado sem implementar matrix
- [ ] `DARE/TASKS.md`: task-010 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-011`
