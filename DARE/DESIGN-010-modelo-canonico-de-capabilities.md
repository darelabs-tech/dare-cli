# DESIGN: Modelo canónico de capabilities (Microplano 010)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/010-modelo-canonico-de-capabilities.md`  
> **Referência:** Microplano 009 (assets) · ADR-007 · DEC-011 · baseline TS 3.18.1  
> **Posição:** 10 de 56  
> **Arquivo:** `DARE/DESIGN-010-modelo-canonico-de-capabilities.md` (não substitui Designs 001–009)  
> **Nota:** Existe implementação parcial em `dare-assets::capability` + `assets/capability-matrix.yml` (~49 entries) — este Design formaliza o contrato MUST, gaps de cobertura Cursor/Codex/Antigravity e política de tipos na crate até 011+.

---

## 1. DESCRIÇÃO

Este Design cobre o **modelo canónico de capabilities** — a fonte única que descreve workflows/comandos/skills DARE e os caminhos de saída por harness (Claude, Cursor, Codex, Antigravity). O problema: cada IDE espalhava cópias divergentes de comandos; sem matriz tipada, adapters (011–014) inventam nomes e frontmatter. A matriz `capability-matrix.yml` + tipos Rust (`Capability`, `HarnessOutputs`, exceções) permitem validar ids, duplicidade de paths e gerar markdown **reproduzível** a partir de uma capability.

A entrega é inventário validável embutido em `assets/` (009), API de load/validate/render, CLI `dare capabilities validate`, documentação DEC-011, e classificação explícita do que ainda não cobre (Cursor rules 25, Agent Skills 48) via exceptions ou backlog. Quem consome são adapters de harness e o utilizador final que instala comandos IDE consistentes.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Matriz tipada v1 | `CapabilityMatrix.version == 1` parseável | Sempre |
| O-02 | Cobertura Claude commands | Contagem `capabilities` com `outputs.claude` == ficheiros baseline `.claude/commands` no inventário | **49** (ou exception documentada) |
| O-03 | Validação | `validate_capability_matrix` rejeita id inválido, duplicate id, duplicate output path, campos vazios | 100% testes |
| O-04 | Render reproduzível | Mesmo `Capability` → mesmo markdown (Claude/Cursor skill) bit-igual | Assert golden/unit |
| O-05 | Exceções intencionais | `exceptions[]` com `id` + `reason` para gaps vs microplano (33/25/48) | Doc + DEC-011 |
| O-06 | Embed + verify | Entry `capability-matrix` no `assets/manifest.yml` passa verify 009 | Verde |
| O-07 | CLI validate | `dare capabilities validate` exit 0 no happy path | Smoke |
| O-08 | Ralph Loop | fmt/clippy/test | 0 falhas |
| O-09 | Desbloquear 011 | MUST fechados; adapters podem consumir matriz | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Paridade multi-IDE |
| Tech Lead | Time DARE CLI Rust | ADR-007 / DEC-011 |
| Engenheiro CLI | Time implementação | Tipos estáveis para 011–014 |
| Usuário Final | Devs | Comandos/skills consistentes |
| Compatibilidade | Tech Lead | Contagens 49/33/25/48 vs TS |
| Segurança | Tech Lead | Paths seguros; sem secrets nas instructions |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Tipos `Capability`, `HarnessOutputs`, `CapabilityException`, `CapabilityMatrix` | MUST | Serde YAML; campos tipados |
| RF-02 | `assets/capability-matrix.yml` version 1 | MUST | Embutido; no inventário 009 |
| RF-03 | Load `load_capability_matrix_from_str` | MUST | version≠1 → `CoreError::Config` |
| RF-04 | Validate ids | MUST | non-empty; sem espaço; sem `_` (kebab-case-ish); único |
| RF-05 | Validate textos | MUST | `title`, `description`, `instructions` non-empty |
| RF-06 | Validate output paths únicos | MUST | União de claude/cursor/codex/antigravity paths sem duplicados |
| RF-07 | Mapa 49 Claude | MUST | 49 capabilities com `outputs.claude` alinhados a baseline (ou exceptions) |
| RF-08 | `HarnessOutputs` opcional por IDE | MUST | `Option<String>` por harness; ausência = não gera para esse IDE |
| RF-09 | `render_claude_command` / `render_agent_skill` | MUST | Determinístico; frontmatter mínimo documentado |
| RF-10 | `exceptions[]` | MUST | Lista para gaps intencionais (Cursor rules/skills incompletos) |
| RF-11 | CLI `dare capabilities validate` | MUST | Load embed + validate; exit 0 / Config |
| RF-12 | Docs + DEC-011 | MUST | `capabilities-canonical.md` |
| RF-13 | Teste matriz carrega e valida | MUST | Unit no crate |
| RF-14 | Contagens Cursor 33 / rules 25 / skills 48 | SHOULD | Completar matriz **ou** exceptions com reason + Classe C |
| RF-15 | Geração build-time de `assets/capabilities/**` | SHOULD | Script ou teste que materializa renders (adapters 011+ podem escrever no projeto) |
| RF-16 | Mover tipos para `dare-harness` | COULD | **Fora** — DEC-011: permanece em `dare-assets` até 011+ |
| RF-17 | UI / editor visual da matriz | COULD | Fora |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contrato de disco

| Path | Papel |
|------|-------|
| `assets/capability-matrix.yml` | Fonte canónica |
| `assets/manifest.yml` | Hash da matrix (009) |
| `assets/capabilities/**` | Outputs gerados (SHOULD; pode ficar vazio neste ciclo) |
| `.claude/commands/*.md` etc. | Destinos de harness (escritos em 011–014) |

### API pública mínima

```text
Capability { id, title, description, instructions, cli_commands, outputs, assets }
HarnessOutputs { claude, cursor, codex, antigravity }
CapabilityException { id, reason }
CapabilityMatrix { version, exceptions, capabilities }
load_capability_matrix_from_str
validate_capability_matrix
render_claude_command / render_agent_skill
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Render bit-igual cross-OS (LF) | Teste |
| RNF-02 | Performance | Validate 49 entries | < 50 ms |
| RNF-03 | Compatibilidade | Win/macOS/Linux | CI 003 |
| RNF-04 | Observabilidade | Erros Config com id/path | Acionável |
| RNF-05 | Manutenibilidade | Módulo `capability.rs` | Clippy limpo |
| RNF-06 | Dependências | serde_yaml, dare-core; sem ciclo harness↔assets indevido | audit |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar ids/paths da matriz antes de qualquer write de harness | OWASP A03 |
| RS-02 | Instructions sem secrets/tokens; review no inventário | OWASP A02 |
| RS-03 | Paths de output relativos; sem `..` (reutilizar `assert_safe_asset_path` onde aplicável) | Path safety 005/009 |
| RS-04 | `cargo audit` + deny | OWASP A06 |
| RS-05 | Sem secrets no YAML | Supply chain |
| RS-06 | Não executar YAML como código | Injection |
| RS-07 | Duplicate path detection impede overwrite cruzado | Integridade |
| RS-08 | Exceptions não silenciam validate de entries presentes | Defense |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| Tipos | `dare-assets` | workspace (DEC-011) |
| YAML | `serde_yaml` 0.10.4 | workspace |
| Embed | rust-embed via 009 | 8.7.2 |
| CLI | `dare-cli` `capabilities validate` | clap |
| Baseline | npm 3.18.1 + ADR-007 | referência contagens |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Direção | Dados | Responsável |
|---------|------|---------|-------|-------------|
| `assets/` embed | Build | In | matrix YAML | 009/010 |
| Baseline `.claude/commands` | Referência | In | 49 nomes | Compat |
| Adapters 011–014 | Consumidor | Out | Capability → ficheiros IDE | Harness |
| CI 003 | Test | In | validate + tests | Time CLI |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** 009 DONE; ADR-007 aprovado (assumido via DEC-011).
- **Tipos em `dare-assets`** até extração opcional em 011+ (não criar crate só para tipos neste ciclo).
- **Não** implementar install completo de harness (011–014).
- Mensagens en-US; docs pt-BR.
- Breaking de schema matrix ⇒ ADR + bump version.

---

## 10. FORA DO ESCOPO (v1)

- Adapters Claude/Cursor/Codex/Antigravity (011–014).
- `dare update` sync de skills remotos.
- Editor visual / dashboard de capabilities.
- Preencher 100% Cursor rules/skills se baseline repo incompleta — usar **exceptions** + Classe C.
- GraphRAG / MCP (040+ / 052).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Prob. | Impacto | Mitigação |
|---|-------|--------|---------|-----------|
| R-01 | Contagem ≠ 49/33/25/48 | Alta | Médio | Exceptions + doc Classe C |
| R-02 | Drift matrix vs `.claude` | Média | Alto | Teste contagem + CI validate |
| R-03 | Render não reproduzível | Baixa | Médio | Snapshot unit |
| R-04 | Paths inseguros na matrix | Baixa | Alto | Validar com path safety |
| R-05 | Instructions com PII/secrets | Baixa | Alto | Review RS-02 |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-17 priorizados (move crate / UI fora)
- [ ] 49 Claude MUST; 33/25/48 via SHOULD+exceptions aceite
- [ ] Tipos em `dare-assets` (DEC-011) aceite
- [ ] ADR-007 / DEC-011 alinhados
- [ ] RS-01…RS-08 validados
- [ ] Pré-requisito 009 OK
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-010-modelo-canonico-de-capabilities.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `crates/dare-assets/src/capability.rs` | Tipos + validate + render |
| `assets/capability-matrix.yml` | Matriz |
| `docs/compatibility/capabilities-canonical.md` | Compat |
| Microplano cita `dare-harness/src/capability.rs` | Destino futuro 011+ — **não** obrigatório agora |

## Apêndice B — Exemplo Capability (fragmento)

```yaml
- id: dare-blueprint
  title: "dare-blueprint"
  description: "Capability IDE for dare-blueprint"
  instructions: |
    Use the /dare-blueprint slash command when appropriate.
  cli_commands: []
  outputs:
    claude: .claude/commands/dare-blueprint.md
    cursor: .cursor/commands/dare-blueprint.md
    codex: .codex/skills/dare-blueprint/SKILL.md
    antigravity: .antigravity/commands/dare-blueprint.md
  assets: []
```

## Apêndice C — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| Tipos + validate + render | ✅ parcial |
| ~49 entries na matrix | ✅ |
| exceptions preenchidas p/ gaps 33/25/48 | ⚠️ provavelmente vazio |
| assets/capabilities/** gerados | ⚠️ gap SHOULD |
| Docs | ✅ básico |

## Apêndice D — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-010-…`.  
3. `/dare-tasks` → `mp010-*`.  
4. Após closeout → [`011-adapter-claude-code.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/011-adapter-claude-code.md).
