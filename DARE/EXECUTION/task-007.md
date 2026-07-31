# TASK 007: ADR-002 Contrato de saída JSON

> **Complexidade:** MED  
> **Depends on:** task-001  
> **Estimativa:** 1 h

---

## 1. OBJETIVO

Ao final, ADR-002 Accepted define estabilidade de `--json`, writers com keys lexicográficas, breaking rules e preservação de unknown keys em config de disco.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 4 · §5.5 ADR-002

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `docs/adr/ADR-002-contrato-saida-json.md` | Conteúdo Accepted |

---

## 4. IMPLEMENTAÇÃO

### Decisão obrigatória

1. Chaves públicas `--json` = Classe A  
2. Writers: ordenação lexicográfica de keys em cada objeto (golden)  
3. Campo opcional + default seguro = não-breaking  
4. Remoção/renomeação/mudança de tipo = Breaking  
5. Allowlist explícita de campos voláteis (timestamps etc.)  
6. Unknown keys em config disco: preservar (flatten)

Frontmatter `status: Accepted` + 5 headings na ordem.

### Testes esperados

- [ ] Contém `lexicográf` ou `lexicograph` / “ordenação”
- [ ] Contém `Breaking`
- [ ] `status: Accepted`
- [ ] Edge: menciona flatten/unknown keys

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Outputs JSON não devem documentar logging de secrets

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
rg -n "^status: Accepted" docs/adr/ADR-002-contrato-saida-json.md
rg -n "Breaking|flatten|lexic" docs/adr/ADR-002-contrato-saida-json.md
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Sem seção Decisão vazia / “TBD”
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Regras §5.5 cobertas
- [ ] `DARE/TASKS.md`: task-007 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-011`
