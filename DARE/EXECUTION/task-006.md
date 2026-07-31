# TASK 006: ADR-001 Compatibilidade de bugs legados

> **Complexidade:** MED  
> **Depends on:** task-001  
> **Estimativa:** 1 h

---

## 1. OBJETIVO

Ao final, `docs/adr/ADR-001-compatibilidade-bugs-legados.md` está `status: Accepted` com classes A/B/C/D, regras D/A e invariantes de segurança RS-06/RS-07.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 4
- **Decisões:** §5.5 ADR-001; Doc Mestre §44; Design Apêndice A

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| MODIFICAR | `docs/adr/ADR-001-compatibilidade-bugs-legados.md` | Conteúdo completo Accepted |

---

## 4. IMPLEMENTAÇÃO

### Frontmatter

`id: ADR-001`, `status: Accepted`, `date`, `deciders`, `tags: [governance]`

### § Decisão (obrigatório)

- Definir classes A/B/C/D com ações preserve/fix/adr_required/must_fix
- Classe D **nunca** preservada por paridade
- Classe A só muda via breaking-change-process
- Referenciar CI-005..CI-014
- Invariantes: path safety; argv separado; redação de secrets em logs/erros

### Headings na ordem

Contexto → Decisão → Consequências → Critérios de aceite → Referências

### Testes esperados

- [ ] Frontmatter `status: Accepted`
- [ ] Contém strings `Classe D` e `path safety` (ou equivalente pt)
- [ ] Cinco headings presentes na ordem
- [ ] Edge: sem `TODO`/`FIXME`

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] RS-06/RS-07 explícitos na Decisão
- [ ] Sem exemplos de exploits além de nomes de classes de vulnerabilidade

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
rg -n "^status: Accepted" docs/adr/ADR-001-compatibilidade-bugs-legados.md
rg -n "^## (Contexto|Decisão|Consequências|Critérios de aceite|Referências)" docs/adr/ADR-001-compatibilidade-bugs-legados.md
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Corpo não pode ter só “placeholder”
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Accepted + conteúdo §5.5
- [ ] `DARE/TASKS.md`: task-006 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-011` (após 007–010 também)
