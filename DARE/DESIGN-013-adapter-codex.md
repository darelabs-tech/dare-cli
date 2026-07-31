# DESIGN: Adapter Codex (Microplano 013)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/013-adapter-codex.md`  
> **Referência:** Microplanos 005, 009, 010, **011–012** (padrão adapter) · ADR-007 · DEC-014 · baseline TS 3.18.1  
> **Posição:** 13 de 56  
> **Arquivo:** `DARE/DESIGN-013-adapter-codex.md` (não substitui Designs 001–012)  
> **Nota:** Existe implementação parcial em `dare-harness::codex` + CLI `dare harness codex` — este Design congela o contrato MUST (detect, `AGENTS.md` com `$skill-name`, skills matrix + `.agents/skills` partilhados, coexistência Antigravity, `UPDATE_HARNESS_IDES`), clarifica exception `agent-skills-full-parity` (48 pacotes ≠ IDE), e lista gaps (help `--force`, smoke, docs, Ralph).

---

## 1. DESCRIÇÃO

Este Design cobre o **adapter Codex** — detecção, instalação e validação de `AGENTS.md`, skills em `.codex/skills/**` (paths `outputs.codex` da matrix) e materialização controlada em `.agents/skills/**` para coexistência com Antigravity. O problema: sem adapter tipado, Codex ficava de fora das políticas de update e duplicava skills divergentes face ao harness Antigravity; a invocação por `$skill-name` precisa estar listada de forma determinística em `AGENTS.md`.

A entrega é a API em `crates/dare-harness/src/codex.rs` (incl. `UPDATE_HARNESS_IDES` com `"codex"`), CLI `dare harness codex {detect|install|validate}`, preserve managed, documentação DEC-014, e classificação do gap “48 Agent Skills pacote” via exception Classe C. Quem consome são developers Codex e, depois, `dare update` (021+) / Antigravity (014).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Detectar Codex | `detect_codex` → `agents_md`, `codex_dir`, `agents_skills` | Unit |
| O-02 | Gerar `AGENTS.md` | Lista `$<id>` por capability com `outputs.codex` | Contém `$dare-design` (ex.) |
| O-03 | Instalar skills Codex | Paths `outputs.codex` (hoje **49**) | Assert unit force |
| O-04 | Partilhar `.agents/skills` | Mesmo corpo managed; preserve unmanaged | Unit coexistência |
| O-05 | Validate | Skills + `AGENTS.md` presentes | Exit 0 / Config |
| O-06 | Update policies | `"codex"` ∈ `UPDATE_HARNESS_IDES` | `update_policies_include_codex() == true` |
| O-07 | CLI smoke | detect / install --force / validate | Exit 0 |
| O-08 | Exception 48 | `agent-skills-full-parity` documentada | Mantida |
| O-09 | Path safety | ProjectRoot + SafeRelativePath + atomic_write | 0 escapes |
| O-10 | Ralph Loop | test / clippy / audit / deny | Exit 0 |
| O-11 | Desbloquear 014 | MUST fechados | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Paridade Codex + update |
| Tech Lead | Time DARE CLI Rust | DEC-014; coexistência Antigravity |
| Engenheiro CLI | Time implementação | API estável em `codex.rs` |
| Usuário Final | Devs Codex | `$skill-name` + skills sem perder custom |
| Compatibilidade | Tech Lead | 49 matrix vs 48 package skills Classe C |
| Segurança | Tech Lead | Path jail; sem secrets em skills |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `detect_codex(root)` | MUST | `{ agents_md, codex_dir, agents_skills }` sem writes |
| RF-02 | `generate_agents_md(root, force)` | MUST | `AGENTS.md` managed; lista `- $\`{id}\`` por capability com `outputs.codex`; preserve unmanaged |
| RF-03 | Invocação `$skill-name` | MUST | Doc + linhas em AGENTS.md; exemplo `$dare-design` |
| RF-04 | `install_codex_skills(root, force)` | MUST | Write `outputs.codex` + `.agents/skills/{id}/SKILL.md` (managed); retorna nº escritos em paths matrix |
| RF-05 | Conteúdo skill | MUST | Prefixo managed + `render_agent_skill(cap)` |
| RF-06 | Preserve | MUST | Unmanaged (sem marcador / sem frontmatter `---` tratado como managed-skill) + `!force` → skip |
| RF-07 | Coexistência Antigravity | MUST | Não sobrescrever `.agents/skills` unmanaged; managed idêntico = reuse sem drift |
| RF-08 | `validate_codex_install` | MUST | Todos `outputs.codex` + `AGENTS.md` existem; missing amostra ≤5 |
| RF-09 | Contagem matrix | MUST | force → **49** skills Codex (SoT matrix) |
| RF-10 | `UPDATE_HARNESS_IDES` | MUST | Inclui `"codex"`; `update_policies_include_codex() == true` |
| RF-11 | CLI `dare harness codex detect\|install\|validate` | MUST | Mensagens en-US; help `--force` overwrite unmanaged |
| RF-12 | Docs DEC-014 | MUST | `docs/compatibility/harness-codex.md` |
| RF-13 | Testes unitários | MUST | Roundtrip; `$skill` em AGENTS.md; coexistência preserve; policies |
| RF-14 | Smoke CLI | MUST | tempdir install --force → validate 49 |
| RF-15 | Exception `agent-skills-full-parity` | MUST | Mantida: 48 package skills ≠ IDE capabilities (044/skill registry) |
| RF-16 | Wire real em `dare update` | SHOULD | Constante pronta; wiring completo no microplano 021+ |
| RF-17 | Paths `.codex` só (sem `.agents`) | COULD | Fora — coexistência exige share |
| RF-18 | Adapter Antigravity completo | COULD | Fora — 014 |
| RF-19 | `dare discover` auto-install | COULD | Fora — 018/019 |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contrato de disco

| Path | Papel | Política |
|------|-------|----------|
| `AGENTS.md` | Índice `$skill-name` | Managed `<!-- dare:managed` |
| `.codex/skills/<id>/SKILL.md` | Skill Codex (matrix `outputs.codex`) | Managed |
| `.agents/skills/<id>/SKILL.md` | Skill partilhada (Antigravity) | Mesmo corpo; preserve unmanaged |
| `assets/capability-matrix.yml` | SoT | Exception agent-skills |

### API pública mínima

```text
UPDATE_HARNESS_IDES: &[&str]  // inclui "codex"
CodexDetect { agents_md, codex_dir, agents_skills }
detect_codex(root) -> CoreResult<CodexDetect>
generate_agents_md(root, force) -> CoreResult<()>
install_codex_skills(root, force) -> CoreResult<usize>
validate_codex_install(root) -> CoreResult<usize>
update_policies_include_codex() -> bool
```

### Marcador managed

- `AGENTS.md` / skills: 1ª linha `<!-- dare:managed` **ou** início `---` (frontmatter skill) tratado como managed para preserve (comportamento atual — documentar Classe B se baseline diferir)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Ordem skills = ordem matrix | Re-run estável |
| RNF-02 | Performance | Install 49 + shared | < 3 s tipicamente |
| RNF-03 | Compatibilidade | Win / macOS / Linux | CI 003 |
| RNF-04 | Observabilidade | Erros Config com path; en-US | Acionável |
| RNF-05 | Manutenibilidade | Lógica em `codex.rs`; CLI thin | Clippy limpo |
| RNF-06 | Idempotência | Install 2× force = mesmo resultado | Teste |
| RNF-07 | Padrão 011/012 | Preserve / force / jail | Revisão cruzada |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths relativos antes de write | OWASP A03 · 005 |
| RS-02 | Sem secrets em AGENTS.md / SKILL.md | OWASP A02 |
| RS-03 | Escrita só sob `ProjectRoot` | Path safety 005 |
| RS-04 | `cargo audit` + `cargo deny` | OWASP A06 |
| RS-05 | Sem secrets em código | Supply chain |
| RS-06 | Skills não executadas como shell pelo adapter | Injection |
| RS-07 | `--force` documentado | Integrity |
| RS-08 | `atomic_write` por ficheiro; validate não apaga | Resilience |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-harness` | `0.1.0-alpha.0` |
| Capabilities | `dare-assets` (`render_agent_skill`) | 010 |
| FS | `dare-core` | 005 |
| CLI | `dare-cli` | `harness codex` |
| Baseline | npm 3.18.1 | referência |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Codex / Agent Skills | Consumidor local | Filesystem | Out | AGENTS.md, skills | Utilizador |
| Antigravity (014) | Coexistência | `.agents/skills` | Shared | SKILL.md | Time CLI |
| `capability-matrix.yml` | Embed | In | In | outputs.codex | 010 |
| `dare update` (021+) | Consumidor | Constante IDs | In | UPDATE_HARNESS_IDES | Future |
| CI 003 | Test | cargo | In | unit + smoke | Time CLI |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** 005, 009, 010 (padrão 011–012).
- Não remover `"codex"` de `UPDATE_HARNESS_IDES` sem ADR.
- Não remover exception `agent-skills-full-parity` sem cobertura 48 package skills.
- Mensagens CLI en-US; docs pt-BR OK.
- Sem git commit automático; sem APIs remotas Codex.
- Implementação parcial: **alinhar** ao Design (gaps), não reescrever cosmético.

---

## 10. FORA DO ESCOPO (v1)

- Adapter Antigravity completo (014).
- Registry / publish de skills-pacote (044–045) — distinto de capability IDE.
- Wiring completo de `dare update` (021+) além da constante de políticas.
- Reduzir matrix Codex a 48 paths (ADR).
- `dare discover` auto-install (018/019).
- Release binário completo (015).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Duplicação divergente `.codex` vs `.agents` | Média | Alto | Mesmo `skill_body`; teste coexistência |
| R-02 | Confusão 49 IDE vs 48 package | Alta | Médio | Exception + docs |
| R-03 | Update futuro ignora constante | Baixa | Alto | Teste `update_policies_include_codex`; DEC-014 |
| R-04 | `--force` apaga skill custom | Média | Alto | Help + default preserve |
| R-05 | Frontmatter `---` como managed demais amplo | Baixa | Médio | Documentar; ajustar só com teste |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-19 priorizados (SoT 49; exception 48; share `.agents`)
- [ ] Coexistência Antigravity aceite (sem install Antigravity neste ciclo)
- [ ] `UPDATE_HARNESS_IDES` com codex aceite
- [ ] DEC-014 / `harness-codex.md` alinhados
- [ ] RS-01…RS-08 validados
- [ ] Pré-requisitos 005/009/010 OK
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-013-adapter-codex.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-harness/src/codex.rs` | Adapter Codex |
| `crates/dare-cli/src/main.rs` | `harness codex` |
| `assets/capability-matrix.yml` | outputs.codex + exceptions |
| `docs/compatibility/harness-codex.md` | Compat + DEC-014 |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| detect / generate_agents_md / install / validate | ✅ parcial |
| `$skill-name` em AGENTS.md | ✅ |
| `.agents/skills` + preserve unmanaged | ✅ teste |
| `UPDATE_HARNESS_IDES` + `update_policies_include_codex` | ✅ |
| CLI harness codex | ✅ parcial |
| Help `--force` | ⚠️ alinhar 011/012 |
| Docs harness-codex | ⚠️ stub |
| Smoke CLI | ⚠️ gap |
| Ralph + TASKS-013 | ⚠️ pendente |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-013-adapter-codex.md`.  
3. `/dare-tasks` → `mp013-*`.  
4. Após closeout → [`014-adapter-antigravity.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/014-adapter-antigravity.md).
