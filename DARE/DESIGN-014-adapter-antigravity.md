# DESIGN: Adapter Antigravity (Microplano 014)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/014-adapter-antigravity.md`  
> **Referência:** Microplanos 005, 009, 010, **011–013** (padrão adapter + Codex share) · ADR-007 · DEC-015 · baseline TS 3.18.1  
> **Posição:** 14 de 56  
> **Arquivo:** `DARE/DESIGN-014-adapter-antigravity.md` (não substitui Designs 001–013)  
> **Nota:** Existe implementação parcial em `dare-harness::antigravity` + CLI `dare harness antigravity` — este Design congela o contrato MUST (`.antigravityrules`, commands matrix, `.agents/skills` partilhados com Codex, workflows, frontmatter `name`/`description`), clarifica SoT **49** vs baseline “48 skills” via exception Classe C, e lista gaps (help `--force`, smoke, docs, Ralph).

---

## 1. DESCRIÇÃO

Este Design cobre o **adapter Antigravity** — detecção, instalação e validação de `.antigravityrules`, commands em `.antigravity/commands/**` (`outputs.antigravity`), Agent Skills partilhadas em `.agents/skills/**` (coexistência com Codex 013), e criação de `.agents/workflows/` quando necessário. O problema: sem adapter tipado, Antigravity e Codex divergiam no conteúdo das skills; frontmatter inválido quebrava o harness; workflows ficavam omissos.

A entrega é a API em `crates/dare-harness/src/antigravity.rs`, CLI `dare harness antigravity {detect|install|validate}`, `validate_skill_frontmatter`, documentação DEC-015, e classificação do gap “48 package skills” via exception `agent-skills-full-parity`. Quem consome são developers Antigravity e o pipeline Codex/update (013/021+). Fecha a série de adapters IDE (011–014) antes do release alpha (015).

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Detectar Antigravity | `detect_antigravity` → rules, dir, skills, workflows | Unit |
| O-02 | Gerar `.antigravityrules` | Managed stub; preserve unmanaged | Unit |
| O-03 | Instalar commands | Paths `outputs.antigravity` | **49** force |
| O-04 | Skills partilhadas | `.agents/skills/{id}/SKILL.md` + frontmatter válido | Validate OK |
| O-05 | Workflows | `.agents/workflows/.gitkeep` (ou dir) | Unit |
| O-06 | Frontmatter | `name:` + `description:` non-empty | Unit reject/incomplete |
| O-07 | Coexistência Codex | Install Codex após Antigravity sem corromper validate | Unit |
| O-08 | Exception 48 | `agent-skills-full-parity` documentada | Mantida |
| O-09 | CLI smoke | detect / install --force / validate | Exit 0 |
| O-10 | Ralph Loop | test / clippy / audit / deny | Exit 0 |
| O-11 | Desbloquear 015 | MUST fechados | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Paridade Antigravity + Codex |
| Tech Lead | Time DARE CLI Rust | DEC-015; fechar adapters 011–014 |
| Engenheiro CLI | Time implementação | API em `antigravity.rs` |
| Usuário Final | Devs Antigravity | Rules + skills + workflows |
| Compatibilidade | Tech Lead | 49 matrix vs 48 package Classe C |
| Segurança | Tech Lead | Path jail; frontmatter sem secrets |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | `detect_antigravity(root)` | MUST | `{ antigravityrules, antigravity_dir, agents_skills, agents_workflows }` sem writes |
| RF-02 | `generate_antigravityrules(root, force)` | MUST | `.antigravityrules` managed; preserve unmanaged |
| RF-03 | `ensure_workflows_dir(root, force)` | MUST | Cria `.agents/workflows/.gitkeep` (marcador); preserve se unmanaged |
| RF-04 | `install_antigravity(root, force)` | MUST | Commands `outputs.antigravity` + shared `.agents/skills/{id}/SKILL.md`; retorna nº commands escritos |
| RF-05 | Conteúdo commands | MUST | Prefixo managed + `render_claude_command` (mesmo padrão Cursor) |
| RF-06 | Conteúdo skills | MUST | Prefixo managed + `render_agent_skill` (mesmo corpo Codex) |
| RF-07 | Preserve | MUST | Unmanaged + `!force` → skip; managed marker `<!-- dare:managed` ou `---` |
| RF-08 | `validate_skill_frontmatter(body)` | MUST | Exige `name:` e `description:` non-empty no bloco `---` |
| RF-09 | `validate_antigravity_install` | MUST | Rules + todos commands + skills com frontmatter OK; missing amostra ≤5 |
| RF-10 | Contagem matrix | MUST | force → **49** commands (SoT) |
| RF-11 | Baseline “48 skills” | MUST | Exception `agent-skills-full-parity` Classe C; não forçar 48 paths na matrix |
| RF-12 | Coexistência Codex | MUST | Teste: Antigravity install → Codex install !force → validate Antigravity ainda 49 |
| RF-13 | CLI `dare harness antigravity detect\|install\|validate` | MUST | en-US; help `--force` overwrite unmanaged |
| RF-14 | Ordem install CLI | MUST | rules → workflows → install_antigravity |
| RF-15 | Docs DEC-015 | MUST | `docs/compatibility/harness-antigravity.md` |
| RF-16 | Testes unitários | MUST | Roundtrip; frontmatter; coexistência Codex |
| RF-17 | Smoke CLI | MUST | tempdir install --force → validate 49 |
| RF-18 | Inventário workflows não-vazio | SHOULD | v1 = só `.gitkeep` (paridade TS “empty workflows”) |
| RF-19 | Release binário | COULD | Fora — 015 |
| RF-20 | `dare discover` / `dare update` | COULD | Fora — 018+/021+ |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contrato de disco

| Path | Papel | Política |
|------|-------|----------|
| `.antigravityrules` | Rules dinâmicas | Managed |
| `.antigravity/commands/<id>.md` | Commands Antigravity | Via matrix |
| `.agents/skills/<id>/SKILL.md` | Skills partilhadas Codex | Mesmo corpo; preserve |
| `.agents/workflows/.gitkeep` | Dir workflows | Marcador vazio |
| `assets/capability-matrix.yml` | SoT | Exception agent-skills |

### API pública mínima

```text
AntigravityDetect { antigravityrules, antigravity_dir, agents_skills, agents_workflows }
detect_antigravity(root) -> CoreResult<AntigravityDetect>
generate_antigravityrules(root, force) -> CoreResult<()>
ensure_workflows_dir(root, force) -> CoreResult<()>
install_antigravity(root, force) -> CoreResult<usize>
validate_skill_frontmatter(body: &str) -> CoreResult<()>
validate_antigravity_install(root: &ProjectRoot) -> CoreResult<usize>
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Ordem = matrix | Re-run estável |
| RNF-02 | Performance | Install 49 + skills | < 3 s tipicamente |
| RNF-03 | Compatibilidade | Win / macOS / Linux | CI 003 |
| RNF-04 | Observabilidade | Erros Config com path; en-US | Acionável |
| RNF-05 | Manutenibilidade | Lógica em `antigravity.rs` | Clippy limpo |
| RNF-06 | Idempotência | Install 2× force = mesmo resultado | Teste |
| RNF-07 | Padrão 011–013 | Preserve / force / jail / share | Revisão cruzada |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar paths relativos antes de write | OWASP A03 · 005 |
| RS-02 | Sem secrets em rules / commands / skills | OWASP A02 |
| RS-03 | Escrita só sob `ProjectRoot` | Path safety 005 |
| RS-04 | `cargo audit` + `cargo deny` | OWASP A06 |
| RS-05 | Sem secrets em código | Supply chain |
| RS-06 | Frontmatter parseado, não executado | Injection |
| RS-07 | `--force` documentado | Integrity |
| RS-08 | `atomic_write`; validate não apaga | Resilience |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Crate | `dare-harness` | `0.1.0-alpha.0` |
| Capabilities | `dare-assets` | 010 |
| FS | `dare-core` | 005 |
| CLI | `dare-cli` | `harness antigravity` |
| Baseline | npm 3.18.1 | referência 48 skills |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Antigravity IDE | Consumidor local | Filesystem | Out | rules, commands, skills, workflows | Utilizador |
| Codex (013) | Coexistência | `.agents/skills` | Shared | SKILL.md | Time CLI |
| `capability-matrix.yml` | Embed | In | In | outputs.antigravity | 010 |
| CI 003 | Test | cargo | In | unit + smoke | Time CLI |
| Release 015 | Downstream | — | — | Adapters fechados | Time CLI |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** 005, 009, 010; padrão 011–013 (Codex share já existe).
- Não remover exception `agent-skills-full-parity` sem cobertura package skills.
- Não divergir corpo de skill vs Codex (mesmo `render_agent_skill`).
- Mensagens CLI en-US; docs pt-BR OK.
- Sem git commit automático; sem APIs remotas.
- Implementação parcial: **alinhar** gaps, não reescrever cosmético.

---

## 10. FORA DO ESCOPO (v1)

- Pipeline de release nativo alpha (015).
- Registry / publish skills-pacote (044–045).
- Popular `.agents/workflows` com workflows reais além de `.gitkeep`.
- `dare discover` / `dare update` orquestração (018+/021+).
- Alterar matrix de 49→48 (ADR).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Drift skill Antigravity vs Codex | Média | Alto | Mesmo `skill_body`; teste coexistência |
| R-02 | Confusão 49 vs 48 | Alta | Médio | Exception + docs |
| R-03 | Frontmatter frágil (só prefix match) | Baixa | Médio | Testes reject/incomplete; Classe B se schema evoluir |
| R-04 | `--force` apaga custom | Média | Alto | Help + default preserve |
| R-05 | Workflows vazios vs expectativa user | Baixa | Baixo | Doc SHOULD; `.gitkeep` explícito |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-20 priorizados (SoT 49; exception 48; share Codex)
- [ ] Frontmatter `name`/`description` MUST aceite
- [ ] Workflows = `.gitkeep` aceite (SHOULD conteúdo real fora)
- [ ] DEC-015 / `harness-antigravity.md` alinhados
- [ ] RS-01…RS-08 validados
- [ ] Pré-requisitos 005/009/010/013 OK
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-014-adapter-antigravity.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-harness/src/antigravity.rs` | Adapter Antigravity |
| `crates/dare-cli/src/main.rs` | `harness antigravity` |
| `assets/capability-matrix.yml` | outputs.antigravity + exceptions |
| `docs/compatibility/harness-antigravity.md` | Compat + DEC-015 |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| detect / rules / workflows / install / validate / frontmatter | ✅ parcial |
| Coexistência Codex (teste) | ✅ |
| CLI harness antigravity | ✅ parcial |
| Help `--force` | ⚠️ alinhar 011–013 |
| Docs | ⚠️ stub |
| Smoke CLI | ⚠️ gap |
| Detect/preserve unit extras | ⚠️ gap |
| Ralph + TASKS-014 | ⚠️ pendente |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-014-adapter-antigravity.md`.  
3. `/dare-tasks` → `mp014-*`.  
4. Após closeout → [`015-pipeline-de-release-nativo-alpha.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/015-pipeline-de-release-nativo-alpha.md).
