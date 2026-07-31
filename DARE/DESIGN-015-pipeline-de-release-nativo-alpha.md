# DESIGN: Pipeline de release nativo alpha (Microplano 015)

> **Versão:** v1.0 | **Data:** 2026-07-21 | **Status:** APPROVED  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/015-pipeline-de-release-nativo-alpha.md`  
> **Referência:** Microplano 003 (CI multi-target) · DEC-016 · ADR-008 (a criar/completar) · Documento Mestre (5 targets, SBOM/checksums) · baseline TS 3.18.1 (canal npm ≠ nativo)  
> **Posição:** 15 de 56  
> **Arquivo:** `DARE/DESIGN-015-pipeline-de-release-nativo-alpha.md` (não substitui Designs 001–014)  
> **Nota:** Existe implementação parcial — `.github/workflows/release.yml`, `installers/install.{sh,ps1}`, `scripts/smoke-release-install.*`, stub `docs/compatibility/release-alpha.md`, DEC-016 no decision log. Este Design congela o contrato MUST (tags alpha, 5 targets, tar.gz/zip, SHA256SUMS, SBOM SPDX, assinatura de checksums, installers + smoke) e lista gaps (ADR-008, SBOM mínimo vs tool-generated, runners macOS vs `build.yml`, docs).

---

## 1. DESCRIÇÃO

Este Design cobre o **pipeline de release nativo alpha** do DARE CLI: publicar binários instaláveis **sem npm** a partir de tags GitHub, com empacotamento por target, checksums, SBOM SPDX, assinatura de checksums e installers iniciais (`install.sh` / `install.ps1`). O problema: o rewrite Rust precisa de um canal de distribuição verificável (GitHub Releases) independente do pacote TypeScript `@dewtech/dare-cli`, para developers instalarem `dare` e validarem `--version` numa instalação limpa.

A entrega são o workflow `release.yml` (tag `v*-alpha*`), artefatos por target, meta (`SHA256SUMS`, `SHA256SUMS.sig`, `SBOM.spdx.json`), installers em `installers/`, smoke local de instalação, e documentação DEC-016 / ADR-008. Quem consome são developers alpha, CI de release e operações que publicam prereleases.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Workflow tags alpha | `release.yml` dispara em `v*-alpha*` / `workflow_dispatch` | Presente + dry_run |
| O-02 | Cinco targets | Matrix release produz 5 pacotes | 5/5 |
| O-03 | Empacotamento | Linux/macOS → `.tar.gz`; Windows → `.zip` | Por target |
| O-04 | SHA256SUMS | Ficheiro com hash de todos os archives | Publicado no Release |
| O-05 | SBOM SPDX | `SBOM.spdx.json` (SPDX-2.3) no Release | Presente |
| O-06 | Assinatura checksums | `SHA256SUMS.sig` (cosign keyless/key **ou** skip documentado) | Artefacto + política |
| O-07 | Installers | `install.sh` + `install.ps1` instalam binário e correm `--version` | Smoke exit 0 |
| O-08 | Smoke instalação limpa | Script local com archive + prefix isolado | Exit 0 |
| O-09 | Release GitHub | Tag push → prerelease com assets | Artefacto instalável |
| O-10 | Ralph Loop (código/scripts) | `cargo fmt --check`, clippy, test (workspace) | Exit 0 |
| O-11 | Docs DEC-016 / ADR-008 | Compat doc + ADR | Completos |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Canal alpha sem npm |
| Tech Lead | Time DARE CLI Rust | DEC-016; ADR-008; 5 targets |
| Operações / Release | Quem publica tags | Workflow previsível; dry_run |
| Engenheiro CLI | Time implementação | Installers + smoke |
| Usuário Final | Devs alpha | `install.sh` / `install.ps1` + `--version` |
| Segurança | Tech Lead | Checksums, SBOM, cosign, secrets GITHUB_TOKEN |
| Compatibilidade | Tech Lead | Diff intencional vs canal npm TS |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Workflow `release-alpha` | MUST | `.github/workflows/release.yml`; trigger tags `v*-alpha*` / `v*-alpha.*` + `workflow_dispatch` |
| RF-02 | Dry-run manual | MUST | `workflow_dispatch` com `dry_run=true` (default) builds/pack sem criar Release |
| RF-03 | Matrix 5 targets | MUST | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` |
| RF-04 | Build release | MUST | `cargo build -p dare-cli --release --target …` (cross só onde necessário, ex. linux aarch64) |
| RF-05 | Package por target | MUST | Nome `dare-${VERSION}-${TARGET}.{tar.gz\|zip}`; binário `dare` / `dare.exe` |
| RF-06 | Upload artifacts CI | MUST | Artifact por target + job meta |
| RF-07 | SHA256SUMS | MUST | Hashes SHA-256 de todos `.tar.gz`/`.zip`; ordenação determinística |
| RF-08 | SBOM SPDX-2.3 | MUST | `SBOM.spdx.json` no Release (mínimo válido SPDX; tool-generated SHOULD) |
| RF-09 | Assinar checksums | MUST | Produzir `SHA256SUMS.sig`; cosign keyless/OIDC ou key via secret; **falha soft** documentada se indisponível (alpha) |
| RF-10 | Publicar GitHub Release | MUST | Em tag push: prerelease `true`; assets = packages + meta + installers |
| RF-11 | `installers/install.sh` | MUST | Detecta OS/arch; baixa archive (+ verifica SHA256SUMS se remoto); instala em `$DARE_PREFIX/bin` (default `~/.local/bin`); corre `--version` |
| RF-12 | `installers/install.ps1` | MUST | Windows x64; zip; checksum opcional; instala sob `%LOCALAPPDATA%\dare\bin`; `--version` |
| RF-13 | Variáveis installer | MUST | `DARE_VERSION` ou `DARE_LOCAL_ARCHIVE`; opcional `DARE_REPO`, `DARE_INSTALL_BASE`, `DARE_PREFIX` |
| RF-14 | Smoke instalação limpa | MUST | `scripts/smoke-release-install.sh` e/ou `.ps1`: build local → package → install prefix isolado → `--version` |
| RF-15 | Permissions mínimas | MUST | `contents: write` + `id-token: write` (OIDC cosign); sem secrets em logs |
| RF-16 | Docs `release-alpha.md` | MUST | Targets, naming, installers, smoke, política assinatura |
| RF-17 | ADR-008 | MUST | Decisão canal alpha nativo (vs npm), SBOM mínimo, cosign soft-fail |
| RF-18 | Release notes | MUST | Auto (`generate_release_notes`) **ou** secção curta em docs |
| RF-19 | SBOM via syft/cyclonedx | SHOULD | Substituir stub JSON por tool se estável no MSRV |
| RF-20 | Alinhar runners macOS a 003 | SHOULD | Preferir `macos-13` / `macos-14` como `build.yml` se flaky em `macos-latest` |
| RF-21 | Windows arm64 | COULD | Fora da matrix alpha |
| RF-22 | Homebrew / scoop / winget | COULD | Fora — microplano 053 |
| RF-23 | Self-update no binário | COULD | Fora — 053 |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contrato de artefatos

| Artefacto | Formato | Exemplo |
|-----------|---------|---------|
| Package Unix | tar.gz | `dare-v0.1.0-alpha.1-x86_64-unknown-linux-gnu.tar.gz` |
| Package Windows | zip | `dare-v0.1.0-alpha.1-x86_64-pc-windows-msvc.zip` |
| Checksums | text | `SHA256SUMS` |
| Signature | cosign blob | `SHA256SUMS.sig` |
| SBOM | SPDX JSON | `SBOM.spdx.json` |
| Installers | sh / ps1 | `install.sh`, `install.ps1` |

### Política de assinatura (alpha)

| Cenário | Comportamento |
|---------|---------------|
| Cosign + OIDC / `COSIGN_KEY` | Assina `SHA256SUMS` → `.sig` |
| Cosign indisponível | Escreve `.sig` com mensagem skip; **não** falha o job (documentar em ADR-008) |
| Stable futuro | Assinatura obrigatória (fora deste microplano) |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Nomes de archive e ordem em SHA256SUMS estáveis | Sort por path |
| RNF-02 | Performance | Build matrix fail-fast=false | 5 targets independentes |
| RNF-03 | Disponibilidade | Release só em tag; dry_run default no dispatch | Sem publish acidental |
| RNF-04 | Segurança | Sem tokens em logs; curl/iwr HTTPS | OWASP A02/A09 |
| RNF-05 | Observabilidade | Logs de job claros (target, artifact name) | GH Actions UI |
| RNF-06 | Manutenibilidade | Pins de actions (já em release.yml) | Não floating `@v4` sem patch se possível |
| RNF-07 | Compatibilidade OS | Installers: Linux, macOS, Windows x64 | Smoke local + CI quando aplicável |
| RNF-08 | Idempotência installer | Reinstalar sobrescreve binário no prefix | Exit 0 |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar inputs do installer (`DARE_VERSION`, paths locais); rejeitar OS/arch não suportados | OWASP A03 |
| RS-02 | Não embutir secrets nos archives; checksums públicos apenas | OWASP A02 |
| RS-03 | Release publish só com `GITHUB_TOKEN` do workflow; sem deploy keys em código | OWASP A01 / supply chain |
| RS-04 | Gate de qualidade pré-tag: CI 003 + `cargo audit` / `cargo deny` sem CVE HIGH/CRITICAL | OWASP A06 |
| RS-05 | Secrets (`COSIGN_KEY`, tokens) só via GitHub Secrets / OIDC — nunca em repo | Supply chain |
| RS-06 | Verificar SHA256SUMS no download remoto (MUST em sh; SHOULD/warn em ps1 se falhar) | Tampering |
| RS-07 | Preferir argv/comandos explícitos nos scripts; evitar `eval` de remote não-fixado | Command injection |
| RS-08 | SBOM presente mesmo se mínimo — transparência de componentes | A06 transparency |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / pin |
|--------|-----------|--------------|
| Linguagem | Rust | **1.85.0** (`dtolnay/rust-toolchain@1.85.0`) |
| Binário | `dare-cli` | workspace `0.1.0-alpha.0` (+ tag release) |
| CI | GitHub Actions | `release.yml` |
| Cross | `cross` (linux aarch64) | `cargo install cross --locked` |
| Cache | `Swatinem/rust-cache` | `v2.7.8` |
| Publish | `softprops/action-gh-release` | `v2.2.1` |
| Assinatura | cosign (opcional) | `v2.4.1` download pin |
| Installers | bash / PowerShell | — |
| SBOM | SPDX-2.3 JSON | stub tool `dare-release-alpha` (SHOULD: syft/cyclonedx) |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| GitHub Actions | CI/CD | HTTPS | In/Out | Build, artifacts | Time CLI |
| GitHub Releases | Distribuição | HTTPS | Out | Archives, SHA256SUMS, SBOM, installers | Ops / workflow |
| Sigstore / cosign | Assinatura | OIDC / key | Out | `SHA256SUMS.sig` | Segurança |
| curl / Invoke-WebRequest | Download | HTTPS | In | Archives + checksums | Installers |
| Baseline npm TS | Referência | — | — | Canal paralelo (não substituído aqui) | Product |

---

## 9. RESTRIÇÕES

- **Pré-requisito:** microplano **003** concluído (matrix CI + checksums base).
- **Canal:** apenas **prerelease** alpha; cutover stable = microplano 056.
- **Versão binário vs tag:** tag Git (`v0.1.0-alpha.N`) nomeia archives; versão clap pode permanecer `0.1.0-alpha.0` até bump explícito — documentar em ADR-008 se divergirem.
- **Sem npm** neste pipeline; pacote TS continua legado até cutover.
- **Mensagens** de installer/scripts: en-US preferido (alinhar language-policy).
- **Sem git commit automático** pelo agente; tags humanas.
- Implementação parcial: **alinhar gaps**, não reescrever cosmético.

---

## 10. FORA DO ESCOPO (v1)

- Package managers (Homebrew, Scoop, winget, Chocolatey) — 053.
- Self-update dentro do binário — 053.
- Assinatura obrigatória / notarização Apple / Authenticode Windows.
- SBOM completo de todas as crates transitivas via scanner enterprise (SHOULD se fácil).
- Target Windows arm64 / musl.
- Substituição do canal npm 3.18.1.
- Comandos UX `welcome` / `info` (016/017) — consumidores do binário, não do pipeline.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Cross linux-aarch64 flaky | Média | Alto | Job isolado; fail-fast=false; documentar retry |
| R-02 | `macos-latest` ≠ matrix 003 | Média | Médio | Alinhar a macos-13/14 (RF-20) |
| R-03 | Cosign soft-fail → “assinado” fraco | Alta | Médio | ADR-008: alpha = best-effort; stable endurece |
| R-04 | SBOM stub insuficiente para auditoria | Média | Médio | SHOULD syft/cyclonedx; mínimo SPDX MUST |
| R-05 | Naming archive vs `latest/download` | Média | Alto | Exigir `DARE_VERSION` ou local archive; documentar |
| R-06 | Tag publish acidental | Baixa | Alto | dry_run default no dispatch; só tag cria Release |
| R-07 | Checksum skip no PowerShell | Média | Médio | Warn explícito; preferir fail em sh |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-23 priorizados (5 targets, SHA256SUMS, SBOM, cosign soft, installers, smoke)
- [ ] Política de assinatura alpha aceite
- [ ] Divergência versão clap vs tag documentável em ADR-008
- [ ] Fora de escopo (053/056) aceite
- [ ] RS-01…RS-08 validados
- [ ] Pronto para `/dare-blueprint` → `BLUEPRINT-015-pipeline-de-release-nativo-alpha.md`

---

## Apêndice A — Paths

| Path | Papel |
|------|-------|
| `.github/workflows/release.yml` | Pipeline alpha |
| `.github/workflows/build.yml` | Referência matrix 003 |
| `installers/install.sh` | Installer Unix |
| `installers/install.ps1` | Installer Windows |
| `scripts/smoke-release-install.sh` | Smoke local Unix |
| `scripts/smoke-release-install.ps1` | Smoke local Windows |
| `docs/compatibility/release-alpha.md` | Docs DEC-016 |
| `docs/adr/ADR-008-*.md` | ADR (a completar) |
| `docs/DECISION-LOG.md` | DEC-016 |

## Apêndice B — Estado atual (gap hint)

| Item | Estado |
|------|--------|
| `release.yml` 5 targets + package + publish | ✅ parcial |
| SHA256SUMS + SBOM stub + cosign soft | ✅ parcial |
| `install.sh` / `install.ps1` | ✅ parcial |
| Smoke scripts | ✅ parcial |
| `release-alpha.md` | ⚠️ stub |
| ADR-008 | 🔴 ausente / incompleto |
| SBOM tool-generated | ⚠️ gap (SHOULD) |
| Runners macOS alinhados a 003 | ⚠️ gap (SHOULD) |
| TASKS/DAG/Ralph formal 015 | ⚠️ pendente |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `BLUEPRINT-015-pipeline-de-release-nativo-alpha.md`.  
3. `/dare-tasks` → `mp015-*`.  
4. Após closeout → [`016-comando-welcome.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/016-comando-welcome.md).
