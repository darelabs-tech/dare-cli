# DESIGN: Design determinístico — `dare design` (Microplano 023)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/023-design-deterministico.md`  
> **Referência:** Microplanos **009** (templates/assets) · **010** (capability) · **019** (project root / `DARE/`) · **004** (saída) · **005** (path safety) · Documento Mestre §22 · baseline TS 3.18.1  
> **Posição:** 23 de 56  
> **Arquivo:** `DARE/DESIGN-023-design-deterministico.md`  
> **Escopo deste ciclo apenas:** geração determinística de `DARE/DESIGN.md` **sem IA**. Tudo o que pertence a microplanos posteriores fica em **Fora do Escopo**.

---

## 1. DESCRIÇÃO

Este Design cobre o comando **`dare design`** do CLI nativo na fatia **determinística**: a partir de uma descrição (argumento ou `--interactive`), produzir a estrutura canónica de **`DARE/DESIGN.md`**, com markers de enrichment vazios/prontos para fases futuras, preservação de texto personalizado fora desses markers, capability `dare-design` nos quatro harnesses, e snapshots de regressão.

Resolve a falta de um scaffold reproduzível sem LLM — hoje o fluxo depende sobretudo da skill IDE. Quem usa: developers, agentes via `/dare-design`, e CI que valida a estrutura gerada.

Entrega: `crates/dare-cli/src/commands/design.rs`, assets da capability, testes/snapshots, docs **DEC-024**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Schema de entrada | Tipo/`DesignInput` com descrição (+ título se interactive) | Unit |
| O-02 | Estrutura canónica | `DARE/DESIGN.md` com secções do template DARE | Snapshot |
| O-03 | Determinismo estrutural | Mesmo input → mesmas secções/markers (campos voláteis normalizados nos goldens) | Golden ×2 |
| O-04 | Modo argumento | `dare design "<desc>"` cria/atualiza o ficheiro | Smoke |
| O-05 | Modo interativo | `dare design --interactive` preenche input via stdin | Unit/smoke |
| O-06 | Markers | Markers de enrichment presentes no output | Snapshot |
| O-07 | Preserve | Conteúdo fora dos markers não é apagado em regenerate | Unit |
| O-08 | Capability | `dare-design` nos 4 harnesses | Assert matrix / install |
| O-09 | Snapshots | Fixture(s) golden sob `tests/` | Presente |
| O-10 | Ralph + docs | fmt/clippy/test/audit/deny + `cli-design.md` + DEC-024 | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Entrada do método DARE sem LLM |
| Tech Lead | Time DARE CLI Rust | Escopo estrito ao 023; sem puxar 024+ |
| Engenheiro CLI | Time implementação | `commands/design.rs` |
| Usuário Final | Devs | `dare design` / `--interactive` |
| Agentes IDE | 4 harnesses | Capability `dare-design` |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Schema de entrada | MUST | Descrição obrigatória (non-empty trim); interactive pode pedir título |
| RF-02 | `dare design <descricao>` | MUST | Gera/atualiza `DARE/DESIGN.md` sob project root |
| RF-03 | `dare design --interactive` | MUST | Prompts stdin; sem TTY → erro tipado (Usage ou InvalidInput) |
| RF-04 | Project root | MUST | `dare_project::find_project_root`; ausente → InvalidInput 4 |
| RF-05 | Diretório `DARE/` | MUST | Cria se necessário (jail 005) |
| RF-06 | Estrutura canónica | MUST | Secções alinhadas ao template embed (`assets/templates/DESIGN-template.md`) |
| RF-07 | Placeholders | MUST | Lacunas → `[A definir]`; secções MUST do template não omitidas |
| RF-08 | Markers de enrichment | MUST | Inserir markers geridos (formato alinhado ao Mestre `<!-- AGENT … -->`; detalhe exacto no Blueprint) nas secções enrichable |
| RF-09 | Preservar áreas personalizadas | MUST | Texto **fora** dos markers geridos preservado em regenerate |
| RF-10 | Capability `dare-design` | MUST | Outputs nos 4 harnesses; `cli_commands` inclui `design` se ainda vazio na matrix |
| RF-11 | Assets capability | MUST | `assets/capabilities/dare-design` (ou render via matrix 010 — coerente com adapters) |
| RF-12 | Snapshots | MUST | ≥1 golden de estrutura para input fixo |
| RF-13 | Report human + `--json` | MUST | Resumo en-US; envelope 004; schema report documentado no Blueprint |
| RF-14 | Exit codes | MUST | Mapa 004 (0 sucesso; 2 usage; 3 not found se aplicável; 4 invalid; 5 io; 1 internal) |
| RF-15 | Docs | MUST | `docs/compatibility/cli-design.md` + DEC-024 |
| RF-16 | Cap de tamanho da descrição | SHOULD | Limite documentado; reject se excedido |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare design <descricao>...
dare design --interactive
# + --json / --no-color (004)
```

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Estrutura estável para o mesmo input | Golden |
| RNF-02 | Performance | Geração local tipicamente < 500 ms | Informal |
| RNF-03 | Offline | Sem rede | Unit |
| RNF-04 | Observabilidade | Erros tipados; tracing span `design` | Unit |
| RNF-05 | Manutenibilidade | Lógica em `commands/design.rs` (paths do microplano) | Clippy |
| RNF-06 | Cross-platform | Paths via `SafeRelativePath` | CI 003 |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar descrição e path de escrita (`DARE/DESIGN.md`) | OWASP A03 / 005 |
| RS-02 | Redact em logs; não dumpar descrição completa em tracing default | OWASP A02 / 004 |
| RS-03 | Writes atómicos sob project root | 005 |
| RS-04 | `cargo audit` + `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Sem secrets em código; sem shell; sem rede | Supply chain |
| RS-06 | Cap de bytes na descrição e ao ler DESIGN existente no merge | Availability |
| RS-07 | Markers HTML comment não executáveis | Injection |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | toolchain | **1.85.0** |
| CLI | `dare-cli` + clap **4.5.40** | workspace |
| Root | `dare-project` | 018/019 |
| Path/FS | `dare-core` | 005 |
| Template | `dare-assets` embed | 009 |
| Capability | matrix 010 + harness 011–014 | workspace |
| Saída | OutputRenderer | 004 |
| Testes | tempfile + snapshots/goldens | workspace |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Filesystem | Local | r/w | In/Out | `DARE/DESIGN.md` | CLI |
| stdin | Terminal | — | In | `--interactive` | CLI |
| stdout | Terminal | — | Out | human / JSON | CLI |
| Baseline TS 3.18.1 | Referência | — | In | paridade classificada | Compat |

---

## 9. RESTRIÇÕES

- Pré-requisitos: **009, 010, 019**.
- Contrato de disco deste ciclo: **apenas** `DARE/DESIGN.md`.
- Paths de implementação deste ciclo: `commands/design.rs` + `assets/capabilities/dare-design`.
- Mensagens CLI en-US.
- Sem mudar contratos sem ADR.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo / dono |
|------|----------------|
| `--ai` / `--provider` / enrichment LLM | Microplano **024** (`dare-ai`) |
| Injeção de conteúdo IA nos markers | **024** (023 só coloca markers + preserve) |
| `dare blueprint` / ler path alternativo de design | **025** |
| `dare tasks` / DAG / execute | **026+** |
| `dare ai doctor/run/…` | **050** |
| Init/bootstrap/scaffold de stacks | **046–047** |
| Multi-escrita `DESIGN-Feature-*` / `DESIGN-NNN-*` | Não no contrato de disco do 023 |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Diff vs TS no markdown | Alta | Médio | DEC-024; snapshots nativos como SoT alpha |
| R-02 | Preserve falha e apaga notas do user | Média | Alto | Testes de merge; markers bem delimitados |
| R-03 | Campo data quebra golden | Média | Baixo | Normalizar voláteis nos snapshots |
| R-04 | Interactive sem TTY em CI | Média | Médio | Erro tipado se !TTY |
| R-05 | Escopo vazar para 024 | Média | Médio | Checklist “Fora do Escopo”; review anti-scope-creep |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Escopo = só checklist do microplano 023 (schema, estrutura, interactive, preserve, markers, capability, snapshots)
- [ ] Fora do Escopo deixa 024/025/050 explícitos
- [ ] Contrato `DARE/DESIGN.md` aceite
- [ ] RS-01…RS-07 ok
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-023-design-deterministico.md`

---

## Apêndice A — Paths (023)

| Path | Papel |
|------|-------|
| `crates/dare-cli/src/commands/design.rs` | Comando + geração |
| `crates/dare-cli/src/main.rs` | `Commands::Design` |
| `assets/capabilities/dare-design` | Asset capability |
| `assets/templates/DESIGN-template.md` | Template canónico |
| `assets/capability-matrix.yml` | Entrada `dare-design` |
| `tests/fixtures/design/` ou `tests/golden/` | Snapshots |
| `docs/compatibility/cli-design.md` | Docs |
| `docs/DECISION-LOG.md` | DEC-024 |

## Apêndice B — Gap atual

| Item | Estado |
|------|--------|
| Template DESIGN | ✅ embed/assets |
| Capability na matrix | ✅ (verificar `cli_commands`) |
| `Commands::Design` | 🔴 |
| Markers + preserve | 🔴 |
| Snapshots | 🔴 |
| Docs DEC-024 | 🔴 |

## Apêndice C — Próximas etapas

1. Aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-023-design-deterministico.md`.  
3. `/dare-tasks` → `mp023-*`.  
4. Closeout → microplano **024** (enrichment por IA).
