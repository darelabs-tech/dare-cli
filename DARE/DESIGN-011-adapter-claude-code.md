# DESIGN: Adapter Claude Code (Microplano 011)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  

> **Fonte:** `DARE-RUST-MICRO-PLANOS/011-adapter-claude-code.md`  
> **Referência:** Microplanos 005 (path safety), 009 (assets), 010 (capabilities) · ADR-007 · DEC-012 · baseline TS 3.18.1  
> **Posição:** 11 de 56  
> **Arquivo:** `DARE/DESIGN-011-adapter-claude-code.md` (não substitui Designs 001–010)  
> **Nota:** Existe implementação parcial em `dare-harness::claude` + CLI `dare harness claude` — este Design formaliza o contrato MUST (detect/install/validate, preserve, settings+PostToolUse, 49 commands) e os gaps de harden/paridade a fechar neste ciclo.

---

## 1. DESCRIÇÃO

Este Design cobre o **adapter Claude Code** — a camada que detecta, instala e valida artefatos nativos do Claude Code no projeto do utilizador (`CLAUDE.md`, `.claude/commands/**`, `.claude/settings.json`). O problema: sem adapter tipado e path-safe, a instalação de slash commands divergia entre cópias manuais e a baseline TypeScript; customizações do utilizador eram sobrescritas ou a matriz de 49 capabilities (010) não chegava ao disco de forma idempotente.

A entrega é a API em `crates/dare-harness/src/claude.rs` consumindo `capability-matrix.yml` via `render_claude_command`, CLI `dare harness claude {detect|install|validate}`, política de preserve (`<!-- dare:managed`), hook `PostToolUse` compatível em `settings.json`, e validação de que os 49 paths Claude da matriz existem no projeto. Quem consome são `dare discover`/`dare update` (ciclos posteriores) e o developer que usa Claude Code com o método DARE.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Detectar presença Claude | `detect_claude` reporta `claude_md` + `claude_dir` booleanos | 100% unit |
| O-02 | Instalar 49 commands | `install_commands` escreve paths `outputs.claude` da matriz | **49** ficheiros (ou skip preserve) |
| O-03 | Idempotência | Re-install em managed sem `--force` não corrompe conteúdo; contagem estável | Assert unit |
| O-04 | Preserve unmanaged | Ficheiro sem marcador managed não é sobrescrito sem `--force` | Assert unit (ex.: 48 escritos) |
| O-05 | CLAUDE.md gerado | `generate_claude_md` com prefixo managed; respeita preserve | Unit + smoke |
| O-06 | settings.json + PostToolUse | `write_settings_json` inclui hook PostToolUse; `_dare_managed` | Unit + JSON parse |
| O-07 | Validate vs matriz | `validate_install` exige todos os paths Claude presentes | Exit 0 / Config |
| O-08 | CLI smoke | `dare harness claude detect\|install\|validate` | Exit 0 no happy path |
| O-09 | Path safety | Todas as escritas via `ProjectRoot` + `SafeRelativePath` + `atomic_write` | 0 escapes |
| O-10 | Ralph Loop | fmt / clippy `-D warnings` / test / audit | 0 falhas |
| O-11 | Desbloquear 012 | MUST fechados; padrão de adapter reutilizável | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Paridade Claude Code com baseline 3.18.1 |
| Tech Lead | Time DARE CLI Rust | DEC-012 / ADR-007; padrão para 012–014 |
| Engenheiro CLI | Time implementação | API estável em `dare-harness` |
| Usuário Final | Devs com Claude Code | Slash commands + CLAUDE.md sem perder customizações |
| Compatibilidade | Tech Lead | 49 commands; classificação Classe A/B/C |
| Segurança | Tech Lead | Jail de paths; sem secrets em hooks/logs |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `detect_claude(root)` | MUST | Retorna `{ claude_md, claude_dir }` sem criar ficheiros |
| RF-02 | `generate_claude_md(root, force)` | MUST | Escreve `CLAUDE.md` managed; skip se unmanaged e `!force` |
| RF-03 | `install_commands(root, force)` | MUST | Para cada capability com `outputs.claude`, render + write managed; retorna nº escritos |
| RF-04 | Conteúdo dos commands | MUST | Prefixo `<!-- dare:managed capability=<id> -->` + `render_claude_command` |
| RF-05 | `write_settings_json(root, force)` | MUST | Gera `.claude/settings.json` com `_dare_managed: true` e hook `PostToolUse` |
| RF-06 | Hook PostToolUse | MUST | Matcher/comando documentados; compatíveis com schema Claude Code settings (sem shell concatenado perigoso) |
| RF-07 | Preserve commands | MUST | Sem marcador managed → não overwrite sem `--force` |
| RF-08 | Preserve settings | MUST | Settings existentes sem `_dare_managed` → skip sem `--force` |
| RF-09 | `validate_install(root)` | MUST | Todos os paths Claude da matriz existem como ficheiro; senão `CoreError::Config` listando missing (amostra) |
| RF-10 | Contagem 49 | MUST | Happy path força: install escreve 49; validate retorna 49 (alinhado a 010) |
| RF-11 | CLI `dare harness claude detect` | MUST | Mensagem human com flags booleanos; exit 0 |
| RF-12 | CLI `dare harness claude install [--force]` | MUST | Chama generate_claude_md + install_commands + write_settings_json; reporta nº commands |
| RF-13 | CLI `dare harness claude validate` | MUST | Chama validate_install; exit 0 / Config |
| RF-14 | Path jail | MUST | Nenhum path absoluto ou `..` nos outputs; reutiliza SafeRelativePath |
| RF-15 | Docs DEC-012 | MUST | `docs/compatibility/harness-claude.md` atualizado |
| RF-16 | Testes unitários | MUST | Roundtrip install/validate; preserve unmanaged |
| RF-17 | Golden vs TS 3.18.1 (nomes/paths) | SHOULD | Diff documentado ou fixture de nomes dos 49 files |
| RF-18 | Merge profundo de settings.json do user | SHOULD | v1 = skip ou replace managed; merge field-level = backlog |
| RF-19 | `dare discover` auto-install Claude | COULD | Fora — microplano 018/019 |
| RF-20 | Adapter Cursor/Codex/Antigravity | COULD | Fora — 012–014 |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contrato de disco

| Path | Papel | Política |
|------|-------|----------|
| `CLAUDE.md` | Steering / metodologia no projeto | Managed se 1ª linha `<!-- dare:managed` |
| `.claude/commands/<id>.md` | Slash commands Claude | Managed via matriz; preserve unmanaged |
| `.claude/settings.json` | Permissions + hooks | Managed se `"_dare_managed": true` |
| `assets/capability-matrix.yml` | Fonte (só leitura) | Embed 009/010 — não alterada neste ciclo salvo bug |

### API pública mínima (`dare-harness`)

```text
ClaudeDetect { claude_md, claude_dir }
detect_claude(root) -> CoreResult<ClaudeDetect>
generate_claude_md(root, force) -> CoreResult<()>
install_commands(root, force) -> CoreResult<usize>
write_settings_json(root, force) -> CoreResult<()>
validate_install(root) -> CoreResult<usize>
```

### Marcador managed

- Commands / CLAUDE.md: primeira linha começa com `<!-- dare:managed`
- settings.json: propriedade `_dare_managed: true` (Classe B se baseline TS usar outro marcador — documentar em DEC-012)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Render + write order estável (iteração matriz) | Bit-igual re-run managed |
| RNF-02 | Performance | Install 49 commands em FS local | < 2 s tipicamente |
| RNF-03 | Compatibilidade | Win / macOS / Linux (paths) | CI 003 |
| RNF-04 | Observabilidade | Erros Config com path/id acionável; sem dump de conteúdo secret | Mensagens en-US |
| RNF-05 | Manutenibilidade | Lógica só em `claude.rs`; CLI thin wrapper | Clippy limpo |
| RNF-06 | Idempotência | Install 2× com force = mesmo resultado funcional | Teste |
| RNF-07 | Dependências | `dare-assets`, `dare-core`, `serde_json`; sem ciclo novo | audit + deny |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths relativos da matriz antes de qualquer write | OWASP A03 · 005 |
| RS-02 | Não embutir secrets/tokens em CLAUDE.md, commands ou hook command | OWASP A02 |
| RS-03 | Escrita apenas sob `ProjectRoot` (jail); deny escape | Path safety 005 |
| RS-04 | `cargo audit` + `cargo deny` sem HIGH/CRITICAL novos | OWASP A06 |
| RS-05 | Sem secrets em código/settings gerados | Supply chain |
| RS-06 | Hook PostToolUse: comando fixo/argv-safe; sem interpolação de input do user | Injection |
| RS-07 | `--force` sobrescreve unmanaged — documentar no help (consentimento explícito) | Integrity |
| RS-08 | Falha parcial: atomic_write por ficheiro; validate reporta missing sem apagar resto | Resilience |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust (toolchain pin) | **1.85.0** |
| Crate | `dare-harness` | workspace `0.1.0-alpha.0` |
| Capabilities | `dare-assets` (matrix + render) | 010 / DEC-011 |
| FS | `dare-core` ProjectRoot / atomic_write | 005 |
| JSON settings | `serde_json` | workspace |
| CLI | `dare-cli` clap `harness claude` | en-US help |
| Baseline | npm `@dewtech/dare-cli@3.18.1` | paridade observável |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Claude Code (IDE) | Consumidor local | Filesystem | Out | commands, settings, CLAUDE.md | Utilizador |
| `capability-matrix.yml` | Asset embutido | Embed | In | 49 capabilities + paths | 010 |
| Baseline TS 3.18.1 | Referência | Diff/docs | In | Nomes/paths esperados | Compat |
| Adapters 012–014 | Irmãos | API pattern | — | Mesmo padrão detect/install/validate | Time CLI |
| CI 003 | Test | cargo test | In | Unit + smoke | Time CLI |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** microplanos **005**, **009** e **010** concluídos (matriz 49 + path safety + assets).
- **Não** alterar schema da capability matrix sem ADR + bump (010/ADR-007).
- Mensagens CLI **en-US**; docs de compatibilidade **pt-BR** OK.
- Sem git commit automático; sem chamar APIs remotas Claude.
- Breaking de paths Classe A (CI-002/CI-003) ⇒ processo de breaking-change + ADR.
- Implementação parcial existente deve ser **alinhada** a este Design (gaps fechados), não reescrita cosmética.

---

## 10. FORA DO ESCOPO (v1)

- Adapters Cursor / Codex / Antigravity (012–014).
- `dare discover` / `dare update` orquestrando install (018/019/021).
- Merge field-level sofisticado de `settings.json` do utilizador (RF-18 SHOULD mínimo = preserve/replace).
- Geração de conteúdo semântico rico de CLAUDE.md via LLM (stub managed + pointer a commands basta).
- Publicação de skills-pacote (`dare skill`) — distinto de capability IDE (ADR-007).
- GraphRAG / MCP / dashboard.
- Release binário completo (015) — smoke local + gates Ralph bastam neste ciclo.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Schema `settings.json` Claude Code muda | Média | Médio | Documentar campos mínimos; teste parse; Classe C se drift intencional |
| R-02 | Utilizador perde customização com `--force` | Média | Alto | Help explícito; default preserve; sem force em CI smoke |
| R-03 | Contagem ≠ 49 por gap na matrix | Baixa | Alto | Depende 010 DONE; teste `assert_eq!(n, 49)` |
| R-04 | Drift nomes vs TS baseline | Média | Médio | Fixture de paths + DEC-012 / changelog Classe B/C |
| R-05 | Race se install paralelo no mesmo root | Baixa | Médio | Documentar single-writer; atomic_write por ficheiro |
| R-06 | Hook executa comando inseguro | Baixa | Alto | RS-06: comando fixo echo/reminder; sem shell user-controlled |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-20 priorizados (discover/outros adapters fora)
- [ ] Política preserve + `--force` aceite
- [ ] settings.json + PostToolUse mínimos aceites (sem merge profundo MUST)
- [ ] Contagem 49 alinhada a 010 / ADR-007
- [ ] DEC-012 / `harness-claude.md` alinhados
- [ ] RS-01…RS-08 validados pelo Tech Lead
- [ ] Pré-requisitos 005/009/010 OK
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-011-adapter-claude-code.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-harness/src/claude.rs` | Adapter Claude |
| `crates/dare-cli/src/main.rs` | Subcomandos `harness claude` |
| `assets/capability-matrix.yml` | Fonte de paths/conteúdo |
| `docs/compatibility/harness-claude.md` | Compat + DEC-012 |
| `CLAUDE.md` / `.claude/**` | Destinos no projeto alvo |

## Apêndice B — Exemplo settings gerado (ilustrativo)

```json
{
  "permissions": {
    "allow": ["Bash(git:*)", "Read(DARE/**)", "Write(DARE/**)"]
  },
  "hooks": {
    "PostToolUse": [{
      "matcher": "Write",
      "hooks": [{
        "type": "command",
        "command": "echo \"File saved. Remember Ralph Loop: cargo test --workspace && cargo clippy --all-features -- -D warnings\""
      }]
    }]
  },
  "_dare_managed": true
}
```

## Apêndice C — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| `detect` / `install` / `validate` API | ✅ parcial |
| CLI `dare harness claude` | ✅ parcial |
| Preserve unmanaged commands | ✅ teste existe |
| settings + PostToolUse | ✅ parcial (confirmar smoke CLI chama `write_settings_json`) |
| Docs harness-claude + DEC-012 | ✅ básico |
| Golden paths vs TS 3.18.1 | ⚠️ SHOULD |
| Harden mensagens / help `--force` | ⚠️ gap |
| Ralph + closeout formal TASKS-011 | ⚠️ pendente ciclo DARE |

## Apêndice D — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-011-adapter-claude-code.md`.  
3. `/dare-tasks` → `mp011-*` + `dare-dag-011.yaml`.  
4. Após closeout → [`012-adapter-cursor.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/012-adapter-cursor.md).
