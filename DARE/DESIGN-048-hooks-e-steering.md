# DESIGN: Hooks e steering (Microplano 048)

> **Versão:** v1.0 | **Data:** 2026-07-26 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/048-hooks-e-steering.md`  
> **Referência:** Documento Mestre §37 Ciclo 19 · path safety **005** · process safety **006** · discover install **019** · baseline TS `@dewtech/dare-cli@3.18.1` · skills `dare-hooks` / `dare-steering` · próximo **049**  
> **Posição:** 48 de 56  
> **Arquivo:** `DARE/DESIGN-048-hooks-e-steering.md`  
> **Escopo deste ciclo:** crates **`dare-hooks`** + **`dare-steering`** + CLI **`dare hooks`** / **`dare steering`** · eventos fechados · allowlist de ações · trust gate (`trusted:false` + `--trust`) · idempotência SHA-256 · list/run/validate · frontmatter scope/glob/priority · exclusão obrigatória `.env*` · docs + **DEC-049**.  
> **Não** verificação avançada / bench (**049**). **Não** dashboard/MCP REST steering (**051/052**). **Não** hooks nativos Cursor IDE. DEC proposto: **DEC-049** (DEC-048 = init/bootstrap).

---

## 1. DESCRIÇÃO

Portar o subsistema de **hooks determinísticos** e **steering files** do DARE CLI TypeScript para Rust: eventos fechados com trust gate, allowlist de ações, execução idempotente por SHA-256, e resolução de instruções por escopo/glob/prioridade — sem LLM.

O problema: sem hooks/steering, agentes e pre-commit não têm superfície estável para `on-save` / `pre-commit` nem para aplicar DNA/PATTERNS/steering por ficheiro. Quem usa: desenvolvedores, CI e agentes IDE (PostToolUse → `dare hooks run on-save`). Entrega verificável: duas crates + CLI + fixtures + docs DEC-049.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-hooks` | Member workspace; `cargo test -p dare-hooks` | Exit 0 |
| O-02 | Eventos fechados | Enum/const set; evento desconhecido → exit **2** | Unit + CLI |
| O-03 | Allowlist de ações | Só ações permitidas executam; resto → InvalidInput/exit 2 | Unit |
| O-04 | Trust gate | Default `trusted:false`; `run` sem trust não executa | Integration |
| O-05 | `--trust` / config | `--trust` **ou** `hooks.trusted: true` permite `run` | Unit |
| O-06 | Idempotência SHA-256 | Re-run mesmo input → skip; hash estável | Unit |
| O-07 | CLI hooks | `list` / `run <evento>` / `validate` | CLI smoke |
| O-08 | Crate `dare-steering` | Parse frontmatter + resolve | Exit 0 tests |
| O-09 | Scope/glob/priority | Ordem determinística; glob match; priority tie-break | Unit |
| O-10 | Exclusão `.env*` | Nenhum path `.env*` elegível em steering | Unit security |
| O-11 | CLI steering | `list` / `show <file>` (+ `--json`) | CLI smoke |
| O-12 | Docs + DEC-049 | Docs compat + DECISION-LOG; matriz 048 | Review |
| O-13 | Ralph | clippy/test crates + CLI + `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Paridade Ciclo 19 com TS 3.18.1 |
| Tech Lead | DARE CLI Rust | Crates isoladas; trust gate; DEC-049 |
| Engenheiro | Consumidor | `dare hooks run on-save --file src/x.rs --trust` |
| Agente IDE | Claude/Cursor | PostToolUse / commands chamam hooks |
| Segurança | — | Sem shell concat; `.env*` fora; trusted default false |
| Compat | Baseline TS | Exit 2 em evento/trust inválido; diffs A/B/C |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-hooks` | MUST | Workspace member; deps core/config; **sem** ciclo com `dare-cli` |
| RF-02 | Eventos fechados | MUST | Conjunto congelado no Blueprint (proposta: `on-save`, `on-file-create`, `on-task-complete`, `pre-commit`) |
| RF-03 | Evento desconhecido | MUST | Exit **2** (Mestre §2.2) |
| RF-04 | Allowlist de ações | MUST | Enum fechado de ações; fora → exit 2 / InvalidInput |
| RF-05 | Config `hooks` | MUST | `hooks.trusted` default **false**; respeitar `enabled:false` |
| RF-06 | Trust gate | MUST | `run` requer `trusted==true` **ou** `--trust`; senão não executa |
| RF-07 | `dare hooks list` | MUST | Lista determinística; `--json` schema versionado |
| RF-08 | `dare hooks run <evento>` | MUST | `--file`, `--task`, `--trust`, `--json`; path jail |
| RF-09 | `dare hooks validate` | MUST | Valida config + defs + allowlist; zero writes |
| RF-10 | Idempotência SHA-256 | MUST | Digest canónico; re-run → skip sem side effect duplicado |
| RF-11 | Spawn | MUST | `SafeCommand` argv-only (006); sem shell concat |
| RF-12 | Crate `dare-steering` | MUST | Workspace member |
| RF-13 | Fontes steering | MUST | `DARE/PROJECT-DNA.md`, `DARE/PATTERNS.md` + `.dare/steering/*.md` |
| RF-14 | Frontmatter | MUST | `scope: project\|glob`, `glob`, `priority` |
| RF-15 | Resolução | MUST | `show <file>`: base → project → globs; sort priority + path ASC |
| RF-16 | Exclusão `.env*` | MUST | Basename `.env` / `.env.*` nunca elegível |
| RF-17 | `dare steering list` | MUST | Ordem de precedência; `--json` |
| RF-18 | `dare steering show <file>` | MUST | Blocos aplicáveis; path escape → InvalidInput |
| RF-19 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-20 | Exit codes | MUST | 0 ok; **2** hooks inválido/trust; 3 NotFound; 4 InvalidInput; 5 Io; 1 Internal |
| RF-21 | Capabilities | MUST | `dare-hooks` / `dare-steering` + `cli_commands` |
| RF-22 | Docs + DEC-049 | MUST | Docs + DECISION-LOG + matriz 048 Concluído |
| RF-23 | Fixtures | MUST | Untrusted bloqueia; trust run; `.env` excluído; unknown event exit 2 |
| RF-24 | SoT defs hooks | SHOULD | Embed e/ou `.dare/hooks.yml` — Blueprint escolhe |
| RF-25 | Cache idempotência | SHOULD | `.dare/hooks-idempotency/` com path jail |

### 4.1 Superfície CLI (proposta)

```bash
dare hooks list [--json]
dare hooks run <event> [--file <rel>] [--task <id>] [--trust] [--json] [-d <dir>]
dare hooks validate [--json] [-d <dir>]
dare steering list [--json] [-d <dir>]
dare steering show <file> [--json] [-d <dir>]
```

### 4.2 Trust

| Fonte | Efeito |
|-------|--------|
| Default | `hooks.trusted = false` |
| Config `hooks.trusted: true` | Permite `run` sem flag |
| CLI `--trust` | Permite `run` nesta invocação |
| Ambos ausentes/false | `run` **não** executa (proposta: exit **2** + `HOOKS_TRUST`) |

### Fora de escopo (ver §10)

- HTTP `GET /steering` (**051/052**)
- Bench / mutation / formal (**049**)
- Hooks nativos API Cursor
- Shell arbitrário fora da allowlist

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Listagens e hashes canónicos | Golden |
| RNF-02 | Performance | `list` típico < 500 ms CI | Soft |
| RNF-03 | Portabilidade | Win/macOS/Linux; globs posix-rel | Cross-plat |
| RNF-04 | Observabilidade | Reports camelCase + tracing | Unit |
| RNF-05 | Isolamento | Domínio nas crates; CLI fino | `cargo tree` |
| RNF-06 | Compat | Paridade TS 3.18.1 eventos/trust/exit 2 | Diff DEC |
| RNF-07 | Manutenibilidade | Eventos + allowlist num módulo | Review |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar evento, ações, paths, globs | OWASP A03 |
| RS-02 | Redigir secrets; não dumpar `.env` | OWASP A02 |
| RS-03 | Path jail `SafeRelativePath` / `ProjectRoot` | OWASP A01 |
| RS-04 | `cargo audit` sem HIGH/CRITICAL novas | OWASP A06 |
| RS-05 | Trust gate default false; `--trust` explícito | Mestre / skill |
| RS-06 | Sem secrets em fixtures/código | Supply chain |
| RS-07 | Exclusão obrigatória `.env*` no steering | MUST microplano |
| RS-08 | Spawn SafeCommand argv-only | Process safety 006 |
| RS-09 | Allowlist fecha superfície de ações | Hardening |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | workspace `1.85+` |
| CLI | `clap` | `=4.5.40` |
| Crates novas | `dare-hooks`, `dare-steering` | workspace |
| Core | `dare-core` | workspace |
| Config / contracts | `dare-config`, `dare-contracts` | workspace |
| Hash | `sha2` | pin workspace |
| Frontmatter / glob | A escolher no Blueprint | pin |
| Baseline | `@dewtech/dare-cli` | 3.18.1 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Filesystem projeto | Local | FS | R/W | Defs, steering, cache | Crates |
| `dare.config.json` | Config | JSON | Leitura | `hooks.*` | dare-config |
| Processos allowlisted | Local | argv | Spawn | stdout/stderr truncados | dare-hooks |
| Template pre-commit | Doc | — | Doc | HOOKS-ADAPTER | Docs |
| Baseline TS 3.18.1 | Referência | — | Comp. | Eventos/trust | Compat |
| MCP `/steering` | — | — | — | **Fora** | 051 |

---

## 9. RESTRIÇÕES

- Pré-requisitos **005**, **006**, **019** satisfeitos.
- Exit **2** para hooks inválidos sem DEC breaking.
- Steering **read-only** (sem `steering write`).
- Um DEC (**049**).
- Sem dep de `dare-agent` / GraphRAG / scaffold.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| Bench / mutation / formal / best-of | **049** |
| Dashboard / MCP `GET /steering` | **051/052** |
| Hooks nativos Cursor IDE | HOOKS-ADAPTER adiado |
| Scripts shell arbitrários do user | Allowlist only |
| LLM para gerar steering | Estático |
| Mudanças init/bootstrap | Já **047** |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Inventário TS de eventos/ações incompleto | Alta | Médio | Blueprint congela + Classe B no DEC |
| R-02 | Trust exit ambíguo | Média | Alto | Congelar exit **2** + `HOOKS_TRUST` |
| R-03 | Glob diverge Win/Unix | Média | Médio | Posix paths + testes cross-plat |
| R-04 | Cache idempotência sem bound | Baixa | Médio | Cap/TTL no Blueprint |
| R-05 | Leak `.env` via show | Baixa | Alto | Deny basename antes de read |
| R-06 | Allowlist ainda perigosa | Média | Alto | Ações mínimas v1 (`validate` / `noop`) |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Eventos e allowlist aceites (ou fechados no Blueprint)
- [ ] Trust gate + exit 2 alinhados com Mestre
- [ ] Exclusão `.env*` e path jail validados
- [ ] Duas crates + CLI aprovados
- [ ] Fora de escopo alinhado
- [ ] DEC id **049** confirmado
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-048-hooks-e-steering.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | Hooks = execução trustida; steering = resolve read-only | 🟢 Mestre §37 |
| ambiguity | Lista exacta eventos/ações vs TS | 🔴 Blueprint |
| ambiguity | SoT defs: embed vs `.dare/hooks.yml` | 🟡 Blueprint |
| ambiguity | Untrusted run exit 2 vs 0+skipped | 🟡 proposta exit **2** + `HOOKS_TRUST` |
| gap | Schema JSON reports | 🔴 Blueprint |

### PM

- Aceite v1: unknown event → exit 2; untrusted não executa; `.env*` fora do steering; Ralph verde; DEC-049.
- Allowlist **mínima** preferível a superfície larga.
