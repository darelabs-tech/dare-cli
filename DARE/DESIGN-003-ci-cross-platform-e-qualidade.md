# DESIGN: CI cross-platform e qualidade (Microplano 003)

> **Versão:** v1.0 | **Data:** 2026-07-20 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/003-ci-cross-platform-e-qualidade.md`  
> **Referência:** Documento Mestre (targets mínimos, SBOM/checksums) · workspace 002 (Rust 1.85.0)  
> **Posição:** 3 de 56  
> **Arquivo:** `DARE/DESIGN-003-ci-cross-platform-e-qualidade.md` (não substitui Designs 001/002)

---

## 1. DESCRIÇÃO

Este Design cobre a **CI cross-platform e os gates de qualidade** do DARE CLI nativo em Rust. O microplano 002 entregou workspace, toolchain e um job Ubuntu mínimo (`rust-workspace-002.yml`); ainda falta matriz multi-OS, cache de Cargo, `cargo deny`, smoke do binário em cada artefato e workflows canônicos de PR/build.

A entrega são pipelines GitHub Actions (`ci.yml` para qualidade em PR, `build.yml` para artefatos multi-target), `deny.toml`, cache, uploads de binários temporários e smoke (`dare --version` / `--help`) nos artefatos produzidos. Quem usa são engenheiros e o processo de merge/release alpha; o usuário final ganha confiança de que o binário compila e roda em Linux, macOS e Windows.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | PR bloqueia regressão de qualidade | Job CI falha se fmt, clippy `-D warnings`, test ou audit/deny crítico falhar | 100% dos gates obrigatórios |
| O-02 | Compilar nos 5 targets mínimos | Builds release bem-sucedidos por target | 5/5: linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64 |
| O-03 | Smoke do binário produzido | Após build, executar `--version` (e `--help`) no artifact | Exit 0 em cada OS runner aplicável |
| O-04 | Supply chain básica | `cargo audit` + `cargo deny` no pipeline | Exit 0 (sem HIGH/CRITICAL / deny policy) |
| O-05 | CI performática o bastante | Cache de Cargo (registry/git/target) ativo nos jobs | Hit de cache documentado / steps presentes |
| O-06 | Desbloquear microplano 004 | Checklist MUST do 003 fechado | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Confiança alpha multiplataforma |
| Tech Lead | Time DARE CLI Rust | Gates obrigatórios, targets, deny policy |
| Engenheiro de plataforma / CLI | Time implementação | Workflows estáveis, cache, smoke |
| Usuário Final (indireto) | Devs do CLI | Binários que “simplesmente funcionam” no OS deles |
| Operações / Release | Quem publica Releases | Artefatos + checksums base para canais futuros |
| Segurança | Tech Lead + AppSec | audit/deny, sem secrets em logs de CI |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Workflow de pull request (`ci.yml`) | MUST | Em PR (paths Rust relevantes): fmt `--check`, clippy `-D warnings`, `cargo test --workspace`; falha o check do PR se qualquer gate falhar |
| RF-02 | Incluir `cargo audit` e `cargo deny` na qualidade | MUST | Steps no `ci.yml` (ou job dedicado); `deny.toml` versionado; falha em violação de política / advisory HIGH+ |
| RF-03 | Workflow de build multi-target (`build.yml`) | MUST | Produz binários release para os 5 targets; upload-artifact por target |
| RF-04 | Targets mínimos | MUST | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` |
| RF-05 | Cache de Cargo | MUST | `actions/cache` ou `Swatinem/rust-cache` (ou equivalente) nos jobs de verify/build |
| RF-06 | Artefatos temporários de CI | MUST | Artifacts nomeados por OS/arch (ex. `dare-linux-x64`); retenção curta (ex. 7–14 dias) documentada |
| RF-07 | Smoke test do binário | MUST | Após build (mesmo job ou job dependente): executar `./dare --version` e `./dare --help` (Windows: `dare.exe`) com exit 0 |
| RF-08 | Checksums dos artefatos | SHOULD | Gerar `SHA256SUMS` (ou por-artifact `.sha256`) no job de build e anexar ao upload |
| RF-09 | Evoluir/retirar job mínimo 002 | SHOULD | `rust-workspace-002.yml` substituído por `ci.yml`/`build.yml` **ou** marcado deprecated e desabilitado para evitar duplicação |
| RF-10 | Documentação CI + release notes curtas | MUST | Seção em `docs/compatibility/` ou README: como rodam os workflows, targets, como baixar artifacts |
| RF-11 | Issue/épico rastreável do 003 | SHOULD | Placeholder no DECISION-LOG |
| RF-12 | SBOM (SPDX) | COULD | Gerar SBOM no build (ex. `cargo cyclonedx` / `syft`); obrigatório em ciclos de release posteriores se adiado |

> Prioridades: **MUST** · **SHOULD** · **COULD**

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | Job `ci` (Ubuntu) com cache quente | Orientativo &lt; 15 min; não bloqueia se flaky de rede |
| RNF-02 | Disponibilidade | CI GitHub Actions | Jobs usam runners `ubuntu-latest`, `macos-latest`, `windows-latest` (+ cross se necessário para ARM Linux) |
| RNF-03 | Segurança | Sem secrets em logs; `GITHUB_TOKEN` default | Nenhum token de registry privado neste ciclo |
| RNF-04 | Manutenibilidade | Toolchain = pin 1.85.0 do repo | Mesmo `rust-toolchain.toml` / MSRV do 002 |
| RNF-05 | Observabilidade | Logs de CI claros por gate (fmt/clippy/test/audit/deny/smoke) | Step names estáveis |
| RNF-06 | Confiabilidade | Smoke usa o **mesmo** binário do artifact (não só `cargo run`) | Path do artifact verificado |
| RNF-07 | Compatibilidade | Diferenças de path Windows vs Unix no smoke documentadas | 0 diferença sem classificação |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Inputs de workflow (paths, matrix) fixos no YAML — sem interpolação insegura de input não confiável em shell | OWASP A03 |
| RS-02 | Nenhum secret/PII em artifacts ou logs de smoke | OWASP A02 |
| RS-03 | Permissões GHA mínimas (`contents: read` no CI; write só se upload precisar) | OWASP A01 |
| RS-04 | `cargo audit` + `cargo deny` sem CVE/advisory HIGH/CRITICAL (e políticas deny) | OWASP A06 |
| RS-05 | Secrets só via GitHub Secrets/vars — nenhum valor em YAML | Supply chain |
| RS-06 | Binários smoke não executam com shell concatenado; invocação argv direta | Segurança por padrão |
| RS-07 | Checksums (SHOULD) permitem verificar integridade do artifact baixado | Supply chain |
| RS-08 | Manter convivência segura com `governance-001.yml` (Node) sem misturar tokens | Isolamento de pipelines |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | **1.85.0** (pin existente) |
| Build | Cargo | workspace 002 |
| CI | GitHub Actions | `ubuntu-latest` / `macos-latest` / `windows-latest` |
| Toolchain action | `dtolnay/rust-toolchain` (ou `rust-toolchain.toml` + rustup) | alinhado a 1.85.0 |
| Cache | `Swatinem/rust-cache@v2` (proposta) ou `actions/cache` | pin no Blueprint |
| Audit | `cargo-audit` | versão pinada no Blueprint |
| Deny | `cargo-deny` + `deny.toml` | versão pinada no Blueprint |
| Artifacts | `actions/upload-artifact@v4` | v4 |
| Cross (Linux ARM) | `cross` **ou** runner + qemu **ou** `use-cross` — **definir no Blueprint** | A confirmar |
| Governança Node | `governance-001.yml` | permanece (docs/scripts) |
| Job legado | `rust-workspace-002.yml` | migrar/desligar (RF-09) |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| GitHub Actions | CI/CD | HTTPS | Entrada+saída | Logs, artifacts | Time DARE CLI |
| crates.io / rustup | Deps/toolchain | HTTPS | Entrada | Crates, toolchain | Time DARE CLI |
| RustSec advisory-db | Advisories | Git/HTTPS | Entrada | DB do `cargo audit` | Time DARE CLI |
| GitHub Releases | Release | — | — | Fora (canais alpha oficiais ≥ 015) | — |

---

## 9. RESTRIÇÕES

- **Prazo:** Pré-requisito do microplano 004 (erros/tracing/saída); 002 deve estar DONE.
- **Orçamento:** Minutos GHA do org; evitar matrix excessiva (só 5 targets).
- **Limitações técnicas:**
  - Não implementar assinatura minisign/cosign completa (ADR-008 futuro).
  - Não publicar em GitHub Releases estáveis neste ciclo — só artifacts de CI.
  - Não expandir superfície CLI além do smoke help/version.
  - Cross-compile Linux ARM pode exigir `cross`/qemu — aceitar complexidade no Blueprint.
- **Regulatórias:** Sem compliance extra; LICENSE Apache-2.0 já definida no 002.
- **Idioma:** docs CI em pt-BR; mensagens do binário smoke em en-US.

---

## 10. FORA DO ESCOPO (v1)

- Microplanos 004+ (erros, tracing avançado, path safety, processos, comandos de domínio).
- Matriz de teste golden TypeScript × Rust completa.
- Publicação Homebrew/winget/scoop e instaladores oficiais.
- Assinatura de releases e SBOM obrigatório em release estável (SBOM = COULD aqui).
- Self-update / package managers (053).
- Alterar MSRV/toolchain (salvo hotfix de CI incompatível — via DEC).
- Refatoração de crates de domínio.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Linux ARM cross flaky/lento | Alta | Médio | Usar `cross` documentado ou build nativo em runner ARM se disponível; timeout generoso |
| R-02 | Duplicação `rust-workspace-002` × `ci.yml` | Alta | Baixo | RF-09: desligar ou redirecionar o workflow 002 |
| R-03 | Cache miss + minutos GHA | Média | Médio | `Swatinem/rust-cache`; keys por `Cargo.lock` + OS |
| R-04 | `cargo deny` muito estrito quebra deps transitivas | Média | Alto | Política deny inicial focada em vulnerabilities/licenses conhecidas; adjuntos documentados |
| R-05 | Smoke no Windows (path `.exe`) falha | Média | Médio | Matrix com `shell`/`if: runner.os`; testes explícitos no Blueprint |
| R-06 | Advisory novo quebra CI sem bump | Média | Médio | Processo: bump pin + re-audit (como no 002) |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-12 priorizados (MUST vs SHOULD/COULD)
- [ ] 5 targets confirmados (RF-04)
- [ ] Estratégia Linux ARM (`cross` vs runner) aceita pelo Tech Lead
- [ ] Destino de `rust-workspace-002.yml` (substituir vs deprecate) decidido
- [ ] RS-01…RS-08 validados
- [ ] Fora de escopo alinhado (sem Releases oficiais)
- [ ] Pré-requisito microplano 002 DONE confirmado
- [ ] Pronto para `/dare-blueprint` → `DARE/BLUEPRINT-003-ci-cross-platform-e-qualidade.md`

---

## Apêndice A — Relação com workflows existentes

| Workflow atual | Papel após 003 |
|----------------|----------------|
| `governance-001.yml` | Mantém (docs/scripts Node) |
| `rust-workspace-002.yml` | Substituído / deprecated (RF-09) |
| `ci.yml` (novo) | Qualidade PR: fmt, clippy, test, audit, deny |
| `build.yml` (novo) | Matrix 5 targets + artifacts + smoke + checksums SHOULD |

## Apêndice B — Smoke mínimo

| Comando | Exit | Stdout |
|---------|------|--------|
| `dare --version` | 0 | `dare 0.1.0-alpha.0` (trim) |
| `dare --help` | 0 | contém `Usage` / `--version` |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.
2. `/dare-blueprint` → `DARE/BLUEPRINT-003-ci-cross-platform-e-qualidade.md`.
3. Após closeout: microplano [`004-erros-tracing-e-saida-da-cli.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/004-erros-tracing-e-saida-da-cli.md).
