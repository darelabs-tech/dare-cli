# TASK 002: Scaffold scripts/governance + verify-structure

> **Complexidade:** MED  
> **Depends on:** —  
> **Estimativa:** 1 h

---

## 1. OBJETIVO

Ao final, `node scripts/governance/verify-structure.mjs` valida a lista fixa de paths do BLUEPRINT §5.3 (exit 0 se presentes, exit 1 se faltar) e `verify-all.mjs` propaga esse resultado.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 1
- **Decisões:** Node 20; sem deps runtime; healthcheck Docker usará este script
- **Paths obrigatórios:** BLUEPRINT §5.3 (mín. 15 ficheiros docs)

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `scripts/governance/package.json` | private, type module, engines >=20 |
| CRIAR | `scripts/governance/verify-structure.mjs` | checker de paths |
| CRIAR | `scripts/governance/verify-all.mjs` | orquestra (só structure por agora) |
| CRIAR | `scripts/governance/fixtures/.gitkeep` | dir fixtures |
| CRIAR | `scripts/governance/verify-structure.test.mjs` | testes node:test |

---

## 4. IMPLEMENTAÇÃO

### Assinatura

```js
// verify-structure.mjs
export const REQUIRED_PATHS = [ /* lista BLUEPRINT §5.3 */ ];
export function verifyStructure(repoRoot = resolveRepoRoot()): { ok: boolean; missing: string[]; checked: number }
// CLI: exit 0 + JSON {"ok":true,"checked":N} | exit 1 + missing list
```

`resolveRepoRoot()`: a partir de `import.meta.url` sobe até achar `docs/` + `scripts/`, ou usa `process.cwd()`.

### `verify-all.mjs`

Importa/chama `verifyStructure`; `process.exit(code)`.

### Testes esperados

- [ ] `should_export_at_least_15_required_paths`
- [ ] `should_report_missing_when_file_absent` (temp dir sem docs → missing.length > 0)
- [ ] `should_ok_when_all_present` (skip se task-001 não rodou — documentar)

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Paths via `path.join` — sem shell
- [ ] Não ler/escrever fora do repo root
- [ ] Sem deps npm além de stdlib (audit N/A se zero deps)

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
node --check scripts/governance/verify-structure.mjs
node --check scripts/governance/verify-all.mjs
node --test scripts/governance/verify-structure.test.mjs
# Se docs já existirem (task-001):
node scripts/governance/verify-structure.mjs
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] `REQUIRED_PATHS` não pode ser `[]`
- [ ] Funções não retornam `{}` fixo de sucesso sem checar fs
- [ ] Sem `TODO`/`not implemented`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Gates passaram
- [ ] Exit codes 0/1 corretos
- [ ] `DARE/TASKS.md`: task-002 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-003` (deps 001+002), `task-004` (deps 002)
