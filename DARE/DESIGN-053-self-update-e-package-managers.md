# DESIGN: Self-update e package managers (Microplano 053)

> **Versão:** v1.0 | **Data:** 2026-07-30 | **Status:** APPROVED (blueprint gerado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/053-self-update-e-package-managers.md`  
> **Referência:** Documento Mestre §16.6 Auto-update · §16.5 Instalação · ADR-008 (release alpha / GitHub Releases) · DEC-004 (CI checksums) · DEC-016 / microplano **015** (pipeline release) · `dare guard` assinatura **034** · `dare update` assets (**022**, crate `dare-update` — **não** confundir) · MCP **052** (DEC-053) · baseline TS `@dewtech/dare-cli@3.18.1` · próximo **054** (hardening)  
> **Posição:** 53 de 56  
> **Arquivo:** `DARE/DESIGN-053-self-update-e-package-managers.md`  
> **Escopo deste ciclo:** CLI **`dare self update|rollback|uninstall`** · lock + download temp · checksum + assinatura · troca atômica do binário · rollback · uninstall · packaging **Homebrew** + **WinGet ou Scoop** · testes de upgrade entre releases · docs + **DEC-054**.  
> **Não** alterar `dare update` (assets de projeto). **Não** cutover npm/legado (**056**). **Não** hardening amplo (**054**). DEC proposto: **DEC-054** (DEC-053 = MCP **052**).

---

## 1. DESCRIÇÃO

Completar a distribuição **native-first** do DARE CLI Rust: o usuário atualiza, reverte ou remove o **binário instalado** sem depender de npm/Node, alinhado ao Documento Mestre e ao canal GitHub Releases (ADR-008).

O problema: após o pipeline alpha (**015**), ainda não existe superfície `dare self *` para upgrade seguro (lock, verify, atomic replace, rollback) nem fórmulas/manifests oficiais para package managers. Quem usa: developers em Linux/macOS/Windows que instalaram via release/installer e querem manter o CLI atualizado. Entrega verificável: comando `dare self`, packaging em `packaging/{homebrew,winget|scoop}`, testes de upgrade interrompido / assinatura inválida, e **DEC-054**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Self-update por canal/versão | `dare self update --channel …` / `--version …` | Smoke + unit |
| O-02 | Lock + download temp | Lock file + dir temp; concorrência rejeitada | Unit / integration |
| O-03 | Checksum | SHA-256 bate com `SHA256SUMS` (ou equivalente Release) | Unit |
| O-04 | Assinatura | Assinatura inválida → **não** instala | Security test |
| O-05 | Troca atômica | Binário final só após verify; falha parcial preserva anterior | Integration |
| O-06 | Rollback | `dare self rollback` restaura versão pré-update | Integration |
| O-07 | Uninstall | `dare self uninstall` remove binário (escopo Blueprint) | Smoke |
| O-08 | Homebrew | Tap/fórmula instala versão correta | Fixture / CI dry |
| O-09 | WinGet **ou** Scoop | Um canal Windows MUST (Blueprint escolhe) | Fixture / dry |
| O-10 | Upgrade entre releases | Fixture A→B; interrupt preserva A | Integration |
| O-11 | Separação de `dare update` | Assets de projeto intactos; help distingue `self` | Regression |
| O-12 | Docs + DEC-054 | `docs/compatibility/cli-self-update.md` (+ packaging) + DECISION-LOG | Review |
| O-13 | Ralph close | `cargo fmt --check`, clippy, test, `cargo audit` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Update nativo sem npm |
| Tech Lead | DARE CLI Rust | DEC-054; atomicidade; supply chain |
| Engenheiro | Consumidor | `dare self update --channel beta` |
| Operações / Release | CI | Artefatos Release + packaging |
| Segurança | — | checksum + assinatura; lock; path safety |
| Compat | Baseline / ADR-008 | Sem confundir com `dare update` assets |
| Package maintainers | Homebrew / WinGet / Scoop | Manifests reproduzíveis |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Superfície CLI | MUST | `dare self update`, `dare self rollback`, `dare self uninstall` no help |
| RF-02 | Canais | MUST | `--channel stable` e `--channel beta` (mapeamento alpha/prerelease congelado no Blueprint) |
| RF-03 | Versão explícita | MUST | `--version <semver>` baixa asset dessa tag/Release |
| RF-04 | Defaults | MUST | Sem flags: canal default documentado (proposta Analyst: `beta` enquanto produto alpha; Blueprint confirma) |
| RF-05 | Origem | MUST | GitHub Releases do repo oficial (ADR-008); URL/base configurável só via env documentada (sem hardcode de token) |
| RF-06 | Lock | MUST | Lock exclusivo durante update; segundo processo → exit tipado (Blueprint: 4 ou 5) |
| RF-07 | Download temp | MUST | Download em diretório temporário; limpeza em sucesso e falha |
| RF-08 | Checksum | MUST | Verificar SHA-256 contra manifesto do Release antes de instalar |
| RF-09 | Assinatura | MUST | Verificar assinatura do manifesto/artefato; formato exato (**cosign / minisign / Ed25519**) congelado no Blueprint — alinhado ao que Release publica |
| RF-10 | Assinatura inválida | MUST | Bloqueia instalação; binário atual intacto |
| RF-11 | Troca atômica | MUST | Replace do binário via rename/atomic write platform-safe; sem estado “meio instalado” observável |
| RF-12 | Backup pré-update | MUST | Preservar binário atual para rollback (path sob cache DARE — Blueprint) |
| RF-13 | Interrupt / falha parcial | MUST | Upgrade interrompido preserva versão anterior (critério de aceite do microplano) |
| RF-14 | Rollback | MUST | `dare self rollback` restaura último backup; sem backup → erro tipado |
| RF-15 | Uninstall | MUST | Remove binário gerenciado pelo instalador/self; **não** apaga projetos do usuário; escopo exato (PATH shim, config global) no Blueprint |
| RF-16 | `--dry-run` | SHOULD | Mostra plano (versão atual → alvo, URL asset, ações) sem I/O destrutivo |
| RF-17 | `--yes` / non-interactive | SHOULD | Sem prompt em CI; default interativo pergunta confirmação em TTY |
| RF-18 | Exits estáveis | MUST | Tabela exit codes antes do happy path (0 ok; 2 usage; 3 root/path; 4 invalid input/lock/channel; 5 network/IO; 6 verify/signature — Blueprint congela) |
| RF-19 | Mensagens en-US | MUST | Erros e progresso em inglês; sem secrets/tokens |
| RF-20 | Path / process safety | MUST | Safe paths; spawn com argv separado; sem shell concatenado |
| RF-21 | Homebrew | MUST | Artefatos em `packaging/homebrew` (fórmula/tap) apontando Release + checksum |
| RF-22 | WinGet **ou** Scoop | MUST | Um dos dois em `packaging/winget` **ou** `packaging/scoop` (Blueprint escolhe; o outro COULD) |
| RF-23 | Package managers | MUST | Manifests instalam a versão correta do asset (validação dry / fixture) |
| RF-24 | Upgrade tests | MUST | Testes automatizados cobrem A→B e falha de assinatura / interrupt |
| RF-25 | Separação `dare update` | MUST | `dare update` (assets projeto / `dare-update`) permanece Class A; help deixa explícito que `self` = binário |
| RF-26 | Capability | MUST | Capability nova (ex. `dare-self`) → `cli_commands:["self"]` + manifest hash **ou** id documentado no Blueprint |
| RF-27 | Docs + DEC-054 | MUST | `docs/compatibility/cli-self-update.md` (+ notas packaging) + DEC-054 append-only; matriz 053 Concluído |
| RF-28 | Cross-platform | MUST | Comportamento documentado em Linux, macOS, Windows (I/O, paths, rename) |
| RF-29 | Código | MUST | Implementação principal em `crates/dare-cli/src/commands/self_update.rs` (+ módulos lib se Blueprint extrair crate) |

> **MUST** · **SHOULD** · **COULD**

### Superfície CLI (este ciclo)

```text
dare self update [--channel stable|beta] [--version <semver>] [--dry-run] [--yes]
dare self rollback [--yes]
dare self uninstall [--yes]
```

### Princípio de não-confusão (inegociável)

| Comando | Escopo |
|---------|--------|
| `dare update` | Assets/harness do **projeto** (crate `dare-update`, microplano 022) |
| `dare self update` | **Binário** do CLI instalado (este microplano) |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Segurança | HTTPS only para download | TLS 1.2+ |
| RNF-02 | Segurança | Verify antes de replace | Sem “trust then check” |
| RNF-03 | Performance | Download tipico alpha | Timeout configurável (Blueprint) |
| RNF-04 | Disponibilidade | Offline | Erro tipado; sem corromper install |
| RNF-05 | Observabilidade | Log: canal, versão de/para, asset name; sem token | Human + `--json` SHOULD |
| RNF-06 | Manutenibilidade | Lógica de verify/apply testável sem rede (fixtures) | Unit |
| RNF-07 | Compat | Linux x64/arm64, macOS x64/arm64, Windows x64 | Alinhar 5 targets ADR-008 |
| RNF-08 | Determinismo | Ordem de passos e mensagens estáveis em dry-run | Contract |
| RNF-09 | Supply chain | `cargo audit` sem HIGH/CRITICAL | Ralph |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar canal/versão/flags antes de rede ou I/O | OWASP A03 |
| RS-02 | Não logar tokens GitHub/PAT; redact URLs com query secrets | OWASP A02 |
| RS-03 | Escrita só em paths allowlisted (temp, backup, binário alvo) | OWASP A01 |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL (HTTP client, crypto) | OWASP A06 |
| RS-05 | Secrets/base URLs via env — nunca hardcoded de credenciais | Supply chain |
| RS-06 | Sem shell concatenado; argv separado | Process 006 |
| RS-07 | Assinatura inválida ou checksum mismatch = hard fail | Integrity |
| RS-08 | Lock impede corrida que corrompe binário | Concurrency |
| RS-09 | Uninstall não apaga dados de projeto do usuário | Blast radius |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão / nota |
|--------|------------|---------------|
| Linguagem | Rust | workspace / `rust-toolchain.toml` (MSRV atual) |
| CLI | `dare-cli` + `commands/self_update.rs` | clap |
| HTTP download | reqwest (ou client já no workspace) | pin Blueprint; TLS |
| Crypto / verify | sha2 + (minisign **ou** ed25519-dalek **ou** cosign verify — Blueprint) | alinhar Release **015**/guard **034** |
| FS / atomic | `dare-core` atomic_write / rename patterns | path safety |
| Lock | file lock (fs4 ou equivalente workspace) | |
| Packaging | `packaging/homebrew`, `packaging/winget` **ou** `scoop` | manifests |
| Release source | GitHub Releases | ADR-008 |
| Testes | tempfile, assert_cmd, fixtures HTTP mock | |
| Docs | `docs/compatibility/cli-self-update.md` | + DEC-054 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| GitHub Releases | Distribuição | HTTPS | Download | binário, SHA256SUMS, assinatura | `dare self update` |
| Homebrew | Package manager | formula/tap | Install | URL + sha256 | `packaging/homebrew` |
| WinGet **ou** Scoop | Package manager | manifest | Install | URL + hash | `packaging/winget` **ou** `scoop` |
| Filesystem local | Runtime | FS | R/W | binário, lock, backup | self_update |
| `dare update` (assets) | Interno | — | — | **Não** reutilizar fluxo de assets | Separação RF-25 |
| npm `@dewtech/dare-cli` | Legado | — | — | Fora; cutover **056** | ADR-008 |

---

## 9. RESTRIÇÕES

| Tipo | Descrição |
|------|-----------|
| Pré-requisito | Microplano **015** concluído; release beta/alpha instalável existente (ADR-008) |
| Arquitetural | Sem ciclos de crate; self-update não depende de `dare-server`/`dare-ai` |
| Contrato | Mudanças de CLI/exit codes exigem Class B/C + DEC |
| Plataforma | Atomic replace no Windows pode exigir rename dance / pending delete — documentar |
| Assinatura alpha | ADR-008 permite cosign best-effort; **053** deve endurecer verify para self-update (não aceitar “signing skipped” em MUST — Blueprint confirma política) |
| Escopo | Um package manager Windows MUST (WinGet **ou** Scoop), não obrigatoriamente ambos |

---

## 10. FORA DO ESCOPO (v1 / este microplano)

- Hardening amplo de paridade/segurança (**054**)
- Cutover npm / descontinuação `@dewtech/dare-cli` (**056**)
- Alterar `dare update` (assets de projeto / harness)
- Auto-update em background / daemon / scheduled task
- UI gráfica de update
- Publicação automática no Homebrew core (tap do projeto basta)
- Assinar com HSM/cloud KMS enterprise
- Atualizar skills/IDE harness via `self` (continua em `dare update` / `dare harness`)
- SBOM regenerado neste comando (já no Release **015**)

---

## 11. RISCOS E MITIGAÇÕES

| Risco | Prob. | Impacto | Mitigação |
|-------|-------|---------|-----------|
| Confundir `dare update` × `dare self update` | Alta | Médio | Help, docs, testes de regressão; nomes distintos |
| Assinatura alpha “skipped” enfraquece RF-09 | Alta | Alto | Blueprint: fail-closed se sig ausente/invalid; política explícita vs ADR-008 |
| Replace falha no Windows (arquivo em uso) | Média | Alto | Documentar; retry/rename; teste Windows |
| Download parcial deixa binário quebrado | Média | Alto | Temp + verify + atomic; backup obrigatório |
| Lock esquecido após crash | Média | Médio | Stale lock TTL / `--force-unlock` documentado (SHOULD) |
| Package manager desatualizado vs Release | Média | Médio | Manifests versionados no repo; CI dry-check |
| Canal `stable` vazio enquanto só alpha | Alta | Médio | Mapear `stable`→erro claro **ou** alias temporário (Blueprint) |
| Escopo uninstall agressivo demais | Baixa | Alto | Só binário gerenciado; nunca apagar `~/projetos` |

---

## 12. AMBIGUIDADES PARA O BLUEPRINT (Analyst → Architect)

| # | Tema | Status | Notas |
|---|------|--------|-------|
| A-01 | Formato de assinatura (cosign blob vs minisign vs Ed25519 guard) | 🔴 | Deve casar com artefatos reais do Release |
| A-02 | WinGet **vs** Scoop como MUST Windows | 🟡 | Microplano diz “ou”; escolher um |
| A-03 | Canal default (`beta` vs `stable`) em alpha | 🟡 | Proposta: default `beta` |
| A-04 | Mapa `stable` enquanto não há tag stable | 🔴 | Erro tipado vs redirect |
| A-05 | Crate dedicada `dare-self` vs só `dare-cli` | 🟡 | Microplano aponta `self_update.rs`; extrair se crescer |
| A-06 | Local do backup/rollback (ex. `~/.dare/self/`) | 🟡 | Fora do ProjectRoot do usuário |
| A-07 | Escopo uninstall (shim PATH, config global) | 🟡 | Mínimo = binário |
| A-08 | Tabela final de exit codes | 🔴 | Definir antes do happy path |
| A-09 | Política se `SHA256SUMS.sig` = “signing skipped” | 🔴 | Fail-closed recomendado para self-update |
| A-10 | HTTP client pin (reqwest features) | 🟡 | Auditoria |

---

## 13. CHECKLIST DE APROVAÇÃO

- [x] Escopo `dare self update|rollback|uninstall` aprovado
- [x] Separação explícita de `dare update` (assets) aceita
- [x] Política de assinatura/checksum fail-closed aceita
- [x] Homebrew MUST + WinGet/Scoop (escolha) aceitos
- [x] Fora de escopo (**054**/**056**/npm) aceito
- [x] DEC proposto **DEC-054** (não reutilizar DEC-053)
- [x] Ambiguidades A-01…A-10 suficientes para `/dare-blueprint`
- [x] Pronto para `/dare-blueprint`

---

## 14. PRÓXIMO PASSO

Após aprovação humana deste DESIGN → `/dare-blueprint` (gera `BLUEPRINT-053-*.md`, tasks, DAG).

Próximo microplano na sequência: [`054-hardening-de-paridade-e-seguranca.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/054-hardening-de-paridade-e-seguranca.md).
