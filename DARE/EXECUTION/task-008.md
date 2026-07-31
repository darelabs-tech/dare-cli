# TASK 008: ADR-004 REST compatível e MCP real

> **Complexidade:** MED  
> **Depends on:** task-001  
> **Estimativa:** 0,75 h

---

## 1. OBJETIVO

Ao final, ADR-004 Accepted deixa explícito que REST legado ≠ MCP protocol e proíbe substituição silenciosa.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 4 · §5.5 ADR-004
- **Ciclos futuros:** 051 REST, 052 MCP (só referenciados)

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `docs/adr/ADR-004-rest-compativel-e-mcp-real.md` | Conteúdo Accepted |

---

## 4. IMPLEMENTAÇÃO

### Decisão obrigatória

- `dare-mcp-server` legado = Express REST (não JSON-RPC/stdio/SSE MCP)
- Transportes distintos; sem swap silencioso
- Implementação nos microplanos 051/052
- Alias/wrapper só com janela de transição documentada

Frontmatter Accepted + 5 headings.

### Testes esperados

- [ ] Menciona REST e MCP como distintos
- [ ] Proíbe substituição silenciosa
- [ ] `status: Accepted`
- [ ] Edge: cita 051/052 ou “ciclos posteriores”

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Não expor endpoints de exemplo com auth bypass

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
rg -n "^status: Accepted" docs/adr/ADR-004-rest-compativel-e-mcp-real.md
rg -n "silencios|MCP|REST" docs/adr/ADR-004-rest-compativel-e-mcp-real.md
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Sem Decisão de uma linha genérica
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Semântica travada
- [ ] `DARE/TASKS.md`: task-008 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-011`
