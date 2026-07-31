# TASK 004: Baseline 3.18.1 + verify-baseline (hash real)

> **Complexidade:** HIGH  
> **Depends on:** task-002  
> **Estimativa:** 2 h

---

## 1. OBJETIVO

Ao final, `baseline-manifest.json` contém SHA-256 real do tarball `@dewtech/dare-cli@3.18.1` e `verify-baseline.mjs` retorna exit 0 com `{"ok":true,"matched":true}`.

---

## 2. CONTEXTO

- **Fase no BLUEPRINT:** Fase 2
- **Decisões:** T-02 hash do tarball npm (não do tree extraído); exit codes 0/1/2

---

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `scripts/governance/verify-baseline.mjs` | §5.1 completo |
| CRIAR | `scripts/governance/verify-baseline.test.mjs` | testes |
| CRIAR | `scripts/governance/fixtures/manifest-valid.json` | fixture |
| CRIAR | `scripts/governance/fixtures/manifest-bad-hash.json` | fixture |
| MODIFICAR | `docs/compatibility/baseline-manifest.json` | hash real |
| MODIFICAR | `docs/compatibility/baseline-3.18.1.md` | narrativa + comando |

---

## 4. IMPLEMENTAÇÃO

### Assinatura

```ts
async function verifyBaseline(opts?: {
  manifestPath?: string;
  skipDownload?: boolean;
}): Promise<
  | { ok: true; package_version: "3.18.1"; content_hash: string; matched: true }
  | { ok: false; code: "SCHEMA_INVALID" | "HASH_MISMATCH" | "DOWNLOAD_FAILED" | "VERSION_MISMATCH"; message: string }
>
```

### Validações

1. JSON parseável; `schema_version==="1.0"`
2. `package_name==="@dewtech/dare-cli"`; `package_version==="3.18.1"`
3. `content_hash` `^[a-f0-9]{64}$`; `content_hash_alg==="sha256"`; `source==="npm"`
4. NO_SECRETS: substrings `token=`, `Bearer `, `npm_`, `ghp_`, `AKIA`
5. Hash do `.tgz` === manifesto

### Fontes do tarball (ordem)

1. `process.env.GOVERNANCE_TARBALL_PATH` se ficheiro existe  
2. `npm pack @dewtech/dare-cli@3.18.1` em temp  
3. Download HTTPS de `resolved_url`

### Exit codes

| Code | Caso |
|------|------|
| 0 | matched |
| 1 | SCHEMA_INVALID / VERSION_MISMATCH / secrets |
| 2 | DOWNLOAD_FAILED / HASH_MISMATCH |

### Testes esperados

- [ ] `should_exit_1_on_invalid_schema`
- [ ] `should_exit_2_on_hash_mismatch` (fixture bad-hash)
- [ ] `should_exit_0_on_real_tarball` (integração; skip se offline sem cache)
- [ ] Edge: manifesto ausente → exit 1
- [ ] Edge: registry offline sem tarball → exit 2

---

## 5. CONSIDERAÇÕES DE SEGURANÇA

- [ ] Não logar URLs com query tokens
- [ ] Temp files apagados após hash
- [ ] Manifesto sem secrets

---

## 6. VALIDATION GATES (RALPH LOOP)

```bash
node --test scripts/governance/verify-baseline.test.mjs
node scripts/governance/verify-baseline.mjs
# exit 0; content_hash no manifesto != aaaa...
```

---

## 7. PADRÕES PROIBIDOS (ANTI-STUB / ANTI-MOCK)

- [ ] Hash não pode permanecer `a`×64
- [ ] Função não retorna `ok:true` hardcoded sem calcular hash
- [ ] Sem `TODO`

---

## 8. CRITÉRIOS DE DONE (ANTI-STUB)

- [ ] Hash real documentado e verificável
- [ ] `DARE/TASKS.md`: task-004 → DONE

---

## 9. PRÓXIMA TASK SUGERIDA

`task-012` (após 005 e 011 também)
