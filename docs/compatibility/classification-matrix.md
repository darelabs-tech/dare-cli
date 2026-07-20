# Matriz de classificação de compatibilidade

Classes: **A** (preservar), **B** (corrigir sem ADR), **C** (ADR obrigatória), **D** (must fix — segurança, RS-07).

| item_id | class | summary | action | adr_ref | source |
|---------|-------|---------|--------|---------|--------|
| CI-001 | A | Exit codes públicos | preserve | — | Doc Mestre |
| CI-002 | A | Nomes de comandos e flags públicas | preserve | — | Doc Mestre |
| CI-003 | A | Schemas persistidos (`dare.config.json`, state, DAG) | preserve | — | Doc Mestre |
| CI-004 | A | IDs canônicos | preserve | — | Doc Mestre |
| CI-005 | B | Texto `dare new` no welcome | fix | — | Doc Mestre |
| CI-006 | B | Mojibake / formatação inconsistente | fix | — | Doc Mestre |
| CI-007 | C | Skill update/remove incompletos | adr_required | ADR-001 | Doc Mestre |
| CI-008 | C | Diferenças de JSON / ordenação | adr_required | ADR-002 | Doc Mestre |
| CI-009 | C | Idioma misto PT/EN | adr_required | language-policy, ADR-003 | Doc Mestre |
| CI-010 | D | Path escape / symlink abuse | must_fix | — | Doc Mestre |
| CI-011 | D | Shell concatenado / execução insegura | must_fix | — | Doc Mestre |
| CI-012 | D | Secret leakage em logs/erros | must_fix | — | Doc Mestre |
| CI-013 | D | Extração insegura de arquivo (zip-slip) | must_fix | — | Doc Mestre |
| CI-014 | D | Assinatura ausente/inválida em releases/skills | must_fix | — | Doc Mestre |

## Regras de validação

- Itens **Classe C** exigem `adr_ref` preenchido (CI-007..CI-009).
- Itens **Classe D** exigem correção imediata (`must_fix`); não preservar comportamento inseguro legado.
- Alterações em itens Classe A seguem `breaking-change-process.md`.
