# DESIGN: Adapter Cursor (Microplano 012)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/012-adapter-cursor.md`  
> **Referência:** Microplanos 005, 009, 010, **011** (padrão adapter) · ADR-007 · DEC-013 · baseline TS 3.18.1  
> **Posição:** 12 de 56  
> **Arquivo:** `DARE/DESIGN-012-adapter-cursor.md` (não substitui Designs 001–011)  
> **Nota:** Existe implementação parcial em `dare-harness::cursor` + CLI `dare harness cursor` (detect / `.cursorrules` / commands via matrix / validate). Este Design congela o contrato MUST alinhado ao 011, clarifica contagens **49 matrix vs 33/25 baseline TS** via exceptions Classe C, e define o gap de **rules `.mdc`** + frontmatter + rules condicionais de stack.

---

## 1. DESCRIÇÃO

Este Design cobre o **adapter Cursor** — detecção, instalação e validação dos artefatos Cursor no projeto (`./.cursorrules`, `.cursor/commands/**`, e, quando no escopo, `.cursor/rules/**`). O problema: sem adapter path-safe e com preserve, a paridade multi-IDE (Claude 011 → Cursor 012) diverge; a baseline TypeScript citava **33 commands** e **25 rules**, enquanto a matriz canónica (010) materializa `outputs.cursor` para as **49** capabilities Claude-aligned.

A entrega é a API em `crates/dare-harness/src/cursor.rs` consumindo `capability-matrix.yml`, CLI `dare harness cursor {detect|install|validate}`, política `<!-- dare:managed`, documentação DEC-013, e classificação explícita do gap 33/25 via exceptions já registadas (`cursor-commands-full-parity`, `cursor-rules-full-parity`). Quem consome são developers Cursor e, depois, `dare discover`/`dare update` (018+).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Detectar Cursor | `detect_cursor` → `cursor_dir` + `cursorrules` | Unit 100% |
| O-02 | Gerar `.cursorrules` | Managed stub; preserve unmanaged | Unit |
| O-03 | Instalar commands | Paths `outputs.cursor` da matrix | Contagem = nº de `Some` (hoje **49**) |
| O-04 | Preserve unmanaged | Sem marcador + `!force` → skip | Unit (ex.: N−1) |
| O-05 | Validate commands | Todos paths cursor existem | Exit 0 / Config |
| O-06 | CLI smoke | `dare harness cursor detect\|install\|validate` | Exit 0 |
| O-07 | Exceptions 33/25 | Doc + exceptions YAML Classe C | Mantidas / atualizadas |
| O-08 | Path safety | `ProjectRoot` + `SafeRelativePath` + `atomic_write` | 0 escapes |
| O-09 | Ralph Loop | test / clippy / audit / deny | Exit 0 |
| O-10 | Desbloquear 013 | MUST fechados | 100% MUST |
| O-11 | Rules `.mdc` | Install + frontmatter **ou** exception explícita | MUST se implementado; senão exception + doc |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Paridade Cursor vs Claude / baseline |
| Tech Lead | Time DARE CLI Rust | DEC-013; exceptions 33/25 |
| Engenheiro CLI | Time implementação | API espelhando 011 |
| Usuário Final | Devs Cursor | Commands + rules sem perder customizações |
| Compatibilidade | Tech Lead | Classe B/C vs TS 3.18.1 |
| Segurança | Tech Lead | Path jail; sem secrets em rules |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `detect_cursor(root)` | MUST | `{ cursor_dir, cursorrules }` sem writes |
| RF-02 | `generate_cursorrules(root, force)` | MUST | `.cursorrules` managed; preserve se unmanaged |
| RF-03 | `install_cursor_commands(root, force)` | MUST | Para cada `outputs.cursor` Some: render + write managed; retorna nº escritos |
| RF-04 | Conteúdo commands | MUST | Prefixo `<!-- dare:managed capability=<id> -->` + corpo render (reutilizar `render_claude_command` **ou** helper partilhado documentado) |
| RF-05 | Preserve commands / cursorrules | MUST | Sem marcador → no overwrite sem `--force` |
| RF-06 | `validate_cursor_install(root)` | MUST | Todos paths cursor da matrix existem; missing → Config (amostra ≤5) |
| RF-07 | Contagem matrix | MUST | Happy path force: written == count(`outputs.cursor`); hoje **49** |
| RF-08 | Baseline 33 commands | MUST | Exception `cursor-commands-full-parity` (Classe C) documentada: SoT = matrix 49, não forçar redução a 33 sem ADR |
| RF-09 | CLI `dare harness cursor detect` | MUST | `cursor_dir=` / `cursorrules=` |
| RF-10 | CLI `install [--force]` | MUST | `generate_cursorrules` + `install_cursor_commands` (+ rules se MUST); help `--force` menciona overwrite unmanaged |
| RF-11 | CLI `validate` | MUST | Validate commands (+ rules se instaladas) |
| RF-12 | Docs DEC-013 | MUST | `docs/compatibility/harness-cursor.md` completo |
| RF-13 | Testes unitários | MUST | Roundtrip force; preserve unmanaged command; detect empty |
| RF-14 | Smoke CLI | MUST | tempdir install --force → validate ok |
| RF-15 | `install_cursor_rules` (`.mdc`) | SHOULD | Inventário mínimo **ou** manter exception `cursor-rules-full-parity` + doc “rules deferred” |
| RF-16 | Validar frontmatter `.mdc` | SHOULD | Se rules escritas: campos mínimos (ex. `description`) documentados |
| RF-17 | Rules condicionais de stack | COULD | Geração por stack detectada — fora v1 salvo fixture simples |
| RF-18 | Reduzir matrix cursor a 33 paths | COULD | Fora — exige ADR + migration |
| RF-19 | Adapter Codex / Antigravity | COULD | Fora — 013/014 |
| RF-20 | `dare discover` auto-install | COULD | Fora — 018/019 |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Política 33 / 25 (decisão de produto)

| Item baseline TS | Estado neste ciclo | Ação |
|------------------|-------------------|------|
| 33 Cursor commands | Matrix tem **49** `outputs.cursor` | **SoT = matrix**; exception Classe C explica gap vs “33” legado |
| 25 Cursor rules `.mdc` | Sem rows na matrix; 0 `.mdc` no repo | Exception `cursor-rules-full-parity` **MUST permanecer**; implementaçao rules = SHOULD |

Critério de aceite do microplano (“33 e 25 **ou** exceptions”) → **satisfeito pelas exceptions** se rules não forem entregues neste ciclo; se rules forem entregues, atualizar exception reason ou removê-la.

### Contrato de disco

| Path | Papel | Política |
|------|-------|----------|
| `.cursorrules` | Steering Cursor | Managed se 1ª linha `<!-- dare:managed` |
| `.cursor/commands/<id>.md` | Slash / commands Cursor | Via `outputs.cursor` |
| `.cursor/rules/**/*.mdc` | Rules Cursor | SHOULD; preserve unmanaged |
| `assets/capability-matrix.yml` | SoT (read-only salvo bug) | Exceptions Classe C |

### API pública mínima (`dare-harness`)

```text
CursorDetect { cursor_dir, cursorrules }
detect_cursor(root) -> CoreResult<CursorDetect>
generate_cursorrules(root, force) -> CoreResult<()>
install_cursor_commands(root, force) -> CoreResult<usize>
validate_cursor_install(root) -> CoreResult<usize>
# SHOULD:
# install_cursor_rules(root, force) -> CoreResult<usize>
# validate_mdc_frontmatter(body: &str) -> CoreResult<()>
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Ordem de escrita = ordem da matrix | Re-run estável |
| RNF-02 | Performance | Install ≤49 commands | < 2 s tipicamente |
| RNF-03 | Compatibilidade | Win / macOS / Linux | CI 003 |
| RNF-04 | Observabilidade | Erros Config com path; en-US CLI | Acionável |
| RNF-05 | Manutenibilidade | Lógica em `cursor.rs`; CLI thin | Clippy limpo |
| RNF-06 | Idempotência | Install 2× force = mesmo resultado funcional | Teste |
| RNF-07 | Padrão 011 | Mesmos marcadores / force / jail | Revisão cruzada |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths relativos antes de write | OWASP A03 · 005 |
| RS-02 | Sem secrets em cursorrules / commands / rules | OWASP A02 |
| RS-03 | Escrita só sob `ProjectRoot` | Path safety 005 |
| RS-04 | `cargo audit` + `cargo deny` | OWASP A06 |
| RS-05 | Sem secrets em código | Supply chain |
| RS-06 | Frontmatter / body não executados como código | Injection |
| RS-07 | `--force` documentado (overwrite unmanaged) | Integrity |
| RS-08 | `atomic_write` por ficheiro; validate não apaga | Resilience |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-harness` | `0.1.0-alpha.0` |
| Capabilities | `dare-assets` | 010 / DEC-011 |
| FS | `dare-core` | 005 |
| CLI | `dare-cli` clap | `harness cursor` |
| Baseline | npm 3.18.1 | referência 33/25 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Cursor IDE | Consumidor local | Filesystem | Out | commands, cursorrules, rules | Utilizador |
| `capability-matrix.yml` | Embed | In | In | outputs.cursor | 010 |
| Exception YAML | Governança | — | In | Classe C 33/25 | Compat |
| Adapter 011 | Padrão | — | — | preserve / managed | Time CLI |
| CI 003 | Test | cargo | In | unit + smoke | Time CLI |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** 005, 009, 010 (e padrão 011) concluídos.
- Não alterar schema da matrix sem ADR + bump.
- Não remover exceptions 33/25 sem substituir por cobertura real + testes.
- Mensagens CLI en-US; docs pt-BR OK.
- Sem git commit automático; sem APIs remotas Cursor.
- Implementação parcial: **alinhar** ao Design, não reescrever cosmético.

---

## 10. FORA DO ESCOPO (v1)

- Adapter Codex / Antigravity (013–014).
- Reduzir matrix de 49→33 cursor paths (ADR).
- Inventário completo de 25 rules legado sem fonte em assets (a menos que SHOULD entregue subset documentado).
- Rules condicionais de stack sofisticadas (RF-17).
- `dare discover` / `dare update` orquestração (018+).
- Skills-pacote (`dare skill`) ≠ capability IDE (ADR-007).
- Release binário completo (015).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Confusão 49 vs 33 | Alta | Médio | Doc + exception Classe C; testes assert matrix count |
| R-02 | Rules nunca entregues | Média | Médio | Exception obrigatória; backlog explícito |
| R-03 | `--force` apaga custom | Média | Alto | Help + default preserve |
| R-04 | Render Claude reused for Cursor | Baixa | Baixo | Documentar; Classe B se formato Cursor divergir depois |
| R-05 | Frontmatter `.mdc` inválido quebra Cursor | Média | Médio | Validate SHOULD antes de write |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-20 priorizados (matrix 49 SoT; 33/25 via exceptions)
- [ ] SHOULD rules `.mdc`: incluir neste ciclo **ou** aceitar só exception + docs
- [ ] Preserve + `--force` alinhados ao 011
- [ ] DEC-013 / `harness-cursor.md` alinhados
- [ ] RS-01…RS-08 validados
- [ ] Pré-requisitos 005/009/010/011 OK
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-012-adapter-cursor.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-harness/src/cursor.rs` | Adapter Cursor |
| `crates/dare-cli/src/main.rs` | `harness cursor` |
| `assets/capability-matrix.yml` | outputs.cursor + exceptions |
| `docs/compatibility/harness-cursor.md` | Compat + DEC-013 |
| `.cursorrules` / `.cursor/**` | Destinos no projeto alvo |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| detect / generate_cursorrules / install commands / validate | ✅ parcial (assert 49) |
| CLI detect/install/validate | ✅ parcial |
| Preserve unmanaged tests | ⚠️ gap (só roundtrip force) |
| Help `--force` | ⚠️ alinhar ao 011 |
| Docs harness-cursor | ⚠️ stub |
| Rules `.mdc` + frontmatter | ❌ não implementado |
| Rules condicionais stack | ❌ fora / COULD |
| Smoke CLI | ⚠️ gap |
| Ralph + TASKS-012 | ⚠️ pendente |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design (decidir: rules SHOULD neste ciclo ou só exception).  
2. `/dare-blueprint` → `BLUEPRINT-012-adapter-cursor.md`.  
3. `/dare-tasks` → `mp012-*`.  
4. Após closeout → [`013-adapter-codex.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/013-adapter-codex.md).
