# TASK 009: ADR-006 Compatibilidade e migração Graph DB

> **Complexidade:** MED  
> **Depends on:** task-001  
> **Estimativa:** 0,75 h

---

## 1. OBJETIVO

Ao final, ADR-006 Accepted cobre `.dare/graph.db` / `.dare/graph.json`, BLOB f32 LE, proibição de migração silenciosa e leitura legada obrigatória.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 4 · §5.5 ADR-006

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `docs/adr/ADR-006-compatibilidade-migracao-graph-db.md` | Conteúdo Accepted |

---

## 4. IMPLEMENTAÇÃO

### Decisão obrigatória

- Paths: `.dare/graph.db` (SQLite), `.dare/graph.json`
- Vector BLOB: `f32` little-endian enquanto compat binária exigida
- Migração silenciosa proibida → migration + changelog
- Leitura legada obrigatória enquanto suportado

Frontmatter Accepted + 5 headings.

### Testes esperados

- [ ] Menciona `graph.db` e `graph.json`
- [ ] Menciona `f32` e little-endian (ou “LE”)
- [ ] Proíbe migração silenciosa
- [ ] `status: Accepted`

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Não incluir dumps de grafo com dados sensíveis de exemplo

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
rg -n "^status: Accepted" docs/adr/ADR-006-compatibilidade-migracao-graph-db.md
rg -n "graph\.(db|json)|f32|silencios" docs/adr/ADR-006-compatibilidade-migracao-graph-db.md
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Sem `TODO`; Decisão completa

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] §5.5 ADR-006 coberto
- [ ] `DARE/TASKS.md`: task-009 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-011`
