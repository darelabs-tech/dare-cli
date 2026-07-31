# DESIGN: Workspace Rust e toolchain (Microplano 002)

> **Versão:** v1.0 | **Data:** 2026-07-20 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/002-workspace-rust-e-toolchain.md`  
> **Referência:** Documento Mestre §12 (workspace recomendado) · DEC-001 (gates cargo)  
> **Posição:** 2 de 56  
> **Arquivo:** `DARE/DESIGN-002-workspace-rust-e-toolchain.md` (não substitui `DARE/DESIGN.md` do microplano 001)

---

## 1. DESCRIÇÃO

Este Design cobre a **criação do workspace Cargo nativo** que sustenta toda a reescrita do DARE CLI em Rust. O problema que resolve é a ausência de toolchain e crates: o ciclo 001 entregou governança e baseline TypeScript 3.18.1, mas ainda não existe código Rust compilável nem binário `dare` nativo.

A entrega é um monorepo Cargo com cinco crates iniciais (`dare-cli`, `dare-core`, `dare-contracts`, `dare-config`, `dare-assets`), `rust-toolchain.toml` pinado, MSRV definida, lint/format com deny-warnings, licença/CODEOWNERS/commits, e um binário `dare` que responde `--help` e `--version`. Quem usa são engenheiros e agentes da reescrita; o usuário final do CLI passa a ter o primeiro smoke nativo, ainda sem comandos de domínio.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Workspace Cargo compilável | `cargo build` e `cargo build --release` exit 0 no workspace | 100% crates do escopo |
| O-02 | Toolchain e MSRV reproduzíveis | `rust-toolchain.toml` + política MSRV documentada; `rustc --version` bate o pin | 1 versão pinada |
| O-03 | Binário `dare` utilizável | `dare --help` e `dare --version` exit 0 com saída não vazia | Ambos OK |
| O-04 | Qualidade de código no gate local/CI mínima | `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` | Exit 0 nos três |
| O-05 | Grafo de crates sem ciclos ilegais | Dependências só no fluxo `cli → domínio → core/contracts/config/assets` | 0 ciclos; 0 dep de domínio → `dare-cli` |
| O-06 | Desbloquear microplano 003 | Checklist MUST do 002 fechado | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Primeiro binário nativo; ritmo alpha |
| Tech Lead | Time DARE CLI Rust | Layout de crates, MSRV, regra de dependência |
| Engenheiro de plataforma / CLI | Time implementação | Toolchain, clippy/fmt, smoke `--help`/`--version` |
| Usuário Final (indireto) | Devs `@dewtech/dare-cli` | Ainda não migram; smoke prova viabilidade nativa |
| Operações / Release | Quem publica GitHub Releases | Artefato CI/smoke instalável mínimo neste ciclo |
| Segurança | Tech Lead + AppSec | `cargo audit` path preparado; sem secrets; path safety nos stubs de core |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Criar Cargo workspace na raiz do repositório | MUST | `Cargo.toml` workspace com `members = ["crates/*"]` (ou lista explícita das 5 crates); `cargo metadata` lista as 5 |
| RF-02 | Fixar `rust-toolchain.toml` | MUST | Arquivo com `channel` (versão estável pinada, ex. `1.85.0` — confirmar no blueprint) + components `rustfmt`, `clippy`; `rustup show` usa o pin do projeto |
| RF-03 | Criar crate `dare-cli` (binário `dare`) | MUST | `crates/dare-cli` com `[[bin]] name = "dare"`; depende das libs conforme regra § Doc Mestre 12.1 |
| RF-04 | Criar crate `dare-core` | MUST | Lib com módulos placeholder mínimos alinhados ao papel (erros/fs/paths/processos/tracing) — **sem** lógica de domínio completa; compila e tem ≥1 teste |
| RF-05 | Criar crate `dare-contracts` | MUST | Lib para schemas/compatibilidade; compila; ≥1 teste smoke; **sem** parsers completos de disco (microplano 007+) |
| RF-06 | Criar crate `dare-config` | MUST | Lib para config/migração futura; compila; ≥1 teste; sem loader completo (008) |
| RF-07 | Criar crate `dare-assets` | MUST | Lib para assets/hashes futuros; compila; ≥1 teste; sem inventário completo (009) |
| RF-08 | Configurar rustfmt e clippy com deny warnings | MUST | `rustfmt.toml` e config clippy (`.clippy.toml` e/ou `Cargo.toml`/`[workspace.lints]`); `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings` exit 0 |
| RF-09 | Definir e documentar MSRV | MUST | MSRV explícita em `Cargo.toml` (`rust-version`) **e** em `docs/compatibility/` ou README do workspace; igual ou ≤ canal do toolchain |
| RF-10 | Adicionar LICENSE, CODEOWNERS e convenção de commits | MUST | `LICENSE` (MIT ou Apache-2.0 — alinhar ao pacote npm se existir); `.github/CODEOWNERS`; `CONTRIBUTING.md` ou seção com Conventional Commits / padrão do time |
| RF-11 | Binário responde `--help` e `--version` | MUST | Via `clap`; `--version` imprime versão do package workspace; `--help` lista uso sem panic; exit 0 |
| RF-12 | Atualizar `dare.config.json` para stack Rust | MUST | `backend: "rust-axum"` (ou valor aceito pelo Ralph Loop Rust); gates passam a `cargo build/test/clippy` |
| RF-13 | Artefato CI ou binário smoke instalável | SHOULD | Job mínimo que builda `dare` **ou** artifact do binário debug/release; matriz completa 5 OS = microplano 003 |
| RF-14 | Issue/épico rastreável do microplano 002 | SHOULD | Link no DECISION-LOG ou docs |

> Prioridades: **MUST** (bloqueia v1 deste microplano) · **SHOULD** · **COULD**

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | `cargo build` (debug) do workspace em máquina de desenvolvimento razoável | < 120 s cold (orientativo); não bloqueia se CI OK |
| RNF-02 | Disponibilidade | N/A serviço — binário local | Smoke `--help`/`--version` determinístico |
| RNF-03 | Segurança | Sem secrets em crates; deps auditáveis | `cargo deny`/audit preparados ou documentados; audit completo reforçado no 003 |
| RNF-04 | Segurança | Nenhum shell concatenado nos stubs de processo em `dare-core` | argv separado se houver API stub |
| RNF-05 | Observabilidade | `tracing` disponível como dep de workspace em `dare-core` (subscriber pode ser mínimo no CLI) | Compila; log de smoke opcional |
| RNF-06 | Manutenibilidade | Regra de dependência Doc Mestre §12.1 enforçada | Review + teste/metadata check |
| RNF-07 | Cross-platform | Workspace builda em Windows (dev atual) e Linux (CI mínima se RF-13) | Sem `[build] target` global em `.cargo/config.toml` |
| RNF-08 | Compatibilidade | `--help`/`--version` alinhados ao espírito TS 3.18.1 (Classe A onde aplicável); diferenças documentadas | 0 diferença sem classificação |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | CLI args (`--help`, `--version`) validados via clap; sem parsing manual inseguro | OWASP A03 |
| RS-02 | Nenhum secret/hash de credencial em código; versão e metadados públicos apenas | OWASP A02 |
| RS-03 | Crates de domínio não expõem APIs que bypassem governança de breaking change (sem I/O de contrato ainda) | OWASP A01 / governança |
| RS-04 | Dependências Cargo sem CVE HIGH/CRITICAL conhecidos no momento do merge (`cargo audit` ou equivalente); se tool ausente, documentar gate no 003 | OWASP A06 |
| RS-05 | Secrets só via env; exemplos `.env*` sem valores | Supply chain |
| RS-06 | Não definir `[build] target` global no `.cargo/config.toml` (protege workspace misto futuro WASM+native) | Doc Mestre / skill rust-workspace |
| RS-07 | Stubs de path/process em `dare-core` não introduzem shell concat nem path escape | Classe D / ADR-001 |
| RS-08 | LICENSE e CODEOWNERS presentes para accountability de mudanças em crates sensíveis | Supply chain / processo |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | **MSRV a fixar** (proposta: estável pinada em `rust-toolchain.toml`, alinhar `rust-version`) |
| Workspace | Cargo | edition **2021** (ou 2024 se MSRV permitir — decidir no Blueprint) |
| CLI | `clap` (derive) | versão pinada no Blueprint |
| Erros | `thiserror` (+ `anyhow` só na borda `dare-cli`) | pinadas |
| Logging | `tracing` / `tracing-subscriber` | pinadas |
| Testes CLI | `assert_cmd`, `predicates` | pinadas |
| Baseline referência | `@dewtech/dare-cli` | **3.18.1** (já hasheada no ciclo 001) |
| Governança | ADRs / docs/compatibility | ciclo 001 (imutável neste escopo salvo DEC nova) |
| Ralph Loop (projeto) | `dare.config.json` → `rust-axum` | após RF-12 |
| CI mínima | GitHub Actions | job build/test/fmt/clippy (matriz completa = **003**) |
| Container | Docker governance (001) | permanece; não substitui Cargo |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| rustup / crates.io | Toolchain e deps | HTTPS | Entrada | Toolchain + crates | Time DARE CLI |
| npm baseline 3.18.1 | Referência de paridade | — | Entrada (já registrada) | Hash/manifesto | Governança 001 |
| GitHub Actions | CI | HTTPS | Saída | Logs, artifact binário (SHOULD) | Time DARE CLI |
| crates posteriores (harness, dag, …) | — | — | — | Fora deste ciclo | — |

---

## 9. RESTRIÇÕES

- **Prazo:** Microplano 002 é pré-requisito do 003 (CI cross-platform); não avançar 003 sem workspace compilando.
- **Orçamento de infra:** Sem serviços pagos novos; só GitHub Actions minutos.
- **Limitações técnicas:**
  - Apenas as **5 crates** listadas; não criar `dare-harness`, `dare-dag`, etc. agora.
  - Sem comandos de domínio (`discover`, `execute`, …) — só `--help` / `--version` (e estrutura clap preparada).
  - Sem `[build] target` global.
  - Dependências: `dare-cli` → libs; **nunca** lib → `dare-cli`.
  - Microplano 001 concluído (pré-requisito).
- **Regulatórias:** LICENSE alinhada à distribuição futura; sem telemetria neste ciclo (ADR-011 futuro).
- **Idioma:** docs de governança pt-BR; mensagens CLI novas em **en-US** (language-policy 001).

---

## 10. FORA DO ESCOPO (v1)

- Microplanos 003–056 (CI matriz completa, erros/tracing avançado, path safety completo, processos, contratos persistidos, adapters IDE, comandos…).
- Matriz de 5 targets release (Linux x64/ARM64, macOS Intel/ARM64, Windows x64) — **003**.
- `cargo deny` / `cargo audit` pipeline completo — preferencialmente **003** (002 pode documentar ou smoke local).
- Implementação real de fs seguro, schemas Zod→serde, config loader, asset packing.
- Paridade golden completa `--help` linha-a-linha com TS (smoke suficiente; diferenças classificadas se houver).
- Troca definitiva dos gates Node de governança (`scripts/governance`) — podem coexistir; Ralph do projeto migra para Cargo.
- Cutover npm / self-update / assinatura de releases.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | `dare-core` vira “god crate” | Alta | Alto | Escopo stub + regra 12.1; crates dedicadas já criadas vazias |
| R-02 | MSRV / edition mal escolhida quebra CI futura | Média | Alto | Pin explícito + documentar upgrade path; Tech Lead aprova versão no Blueprint |
| R-03 | Ralph Loop Node vs Cargo conflita após RF-12 | Média | Médio | Atualizar `dare.config.json` + scripts npm podem permanecer para governance-001.yml |
| R-04 | Escopo creep (clap completo de 25 comandos) | Alta | Médio | RF-11 limita a help/version; subcommands vazios proibidos ou `--help` só root |
| R-05 | Windows path / CRLF em rustfmt | Média | Baixo | `rustfmt.toml` com `newline_style = "Unix"`; testar no host Win atual |
| R-06 | LICENSE divergente do npm | Baixa | Médio | Copiar/espelhar LICENSE do pacote de referência ou Doc Mestre |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Requisitos funcionais RF-01–RF-14 revisados e priorizados
- [ ] Requisitos de segurança RS-01–RS-08 validados pelo Tech Lead
- [ ] MSRV / canal do `rust-toolchain.toml` escolhidos (não deixar “[A definir]” no Blueprint)
- [ ] Regra de dependência §12.1 confirmada
- [ ] `dare.config.json` → `rust-axum` aceito (impacto no Ralph Loop)
- [ ] Fora do escopo alinhado (sem crates extras, sem CI matriz 003)
- [ ] Riscos R-01/R-04 com mitigação aceita
- [ ] Pré-requisito microplano 001 confirmado (DONE)
- [ ] Pronto para `/dare-blueprint` **deste** arquivo (`DARE/DESIGN-002-workspace-rust-e-toolchain.md`)

---

## Apêndice A — Crates deste ciclo vs backlog Doc Mestre §12

| Crate | Neste microplano |
|-------|------------------|
| `dare-cli` | **Sim** (bin `dare`) |
| `dare-core` | **Sim** (stub estruturado) |
| `dare-contracts` | **Sim** (stub) |
| `dare-config` | **Sim** (stub) |
| `dare-assets` | **Sim** (stub) |
| `dare-harness` … `dare-telemetry` | Não (ciclos posteriores) |

## Apêndice B — Superfície CLI neste ciclo

| Invocação | Comportamento esperado |
|-----------|------------------------|
| `dare --help` | Usage clap; exit 0 |
| `dare --version` | Versão semver do package; exit 0 |
| Outros subcomandos | Fora de escopo (não implementar ou não anunciar) |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.
2. `/dare-blueprint` apontando para `DARE/DESIGN-002-workspace-rust-e-toolchain.md`.
3. Após aceite: microplano [`003-ci-cross-platform-e-qualidade.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/003-ci-cross-platform-e-qualidade.md).
