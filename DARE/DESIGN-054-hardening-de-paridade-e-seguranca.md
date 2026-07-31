# DESIGN: Hardening de paridade e segurança (Microplano 054)

> **Versão:** v1.0 | **Data:** 2026-07-31 | **Status:** APPROVED (blueprint gerado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/054-hardening-de-paridade-e-seguranca.md`  
> **Referência:** Documento Mestre §42 (paridade TS×Rust) · §42.2 normalizações · §42.4 security tests · §48 métricas · `docs/compatibility/classification-matrix.md` (A/B/C/D) · ADR-001..008 · baseline `@dewtech/dare-cli@3.18.1` · `fixtures-inventory.md` · self-update **053** (DEC-054) · próximo **055** (pilotos / shadow / RC)  
> **Posição:** 54 de 56  
> **Arquivo:** `DARE/DESIGN-054-hardening-de-paridade-e-seguranca.md`  
> **Escopo deste ciclo:** golden suite completa · comparação observável (exit/stdout/stderr/tree/content/DB/state/HTTP) · normalizações permitidas · fuzzing parsers/paths · security suite (injection, env leak, archive traversal, signature mismatch) · baselines de startup/memória/tamanho · resolver **ou** documentar cada diferença · docs + **DEC-055**.  
> **Não** pilotos shadow / RC (**055**). **Não** cutover npm (**056**). **Não** mudar contrato Classe A sem ADR. DEC proposto: **DEC-055** (DEC-054 = self-update **053**).

---

## 1. DESCRIÇÃO

Fechar o ciclo de **confiança pré-RC** do DARE CLI Rust: provar, de forma automatizada e auditável, que o comportamento observável bate com a baseline TypeScript **3.18.1** (ou está classificado) e que as superfícies críticas de segurança (paths, processos, archives, assinaturas, secrets) não têm vulnerabilidade crítica aberta.

O problema: comandos e crates já existem, mas ainda faltam a **golden suite** consolidada, a **security suite** regressiva e a medição formal de startup/memória/tamanho — sem isso, 055 (pilotos) e 056 (cutover) arriscam descobrir regressões em produção. Quem usa: Tech Lead / Release / Segurança antes do freeze de contrato. Entrega verificável: `tests/golden`, `tests/security`, `tests/cross-platform`, relatório de diferenças classificadas, baselines de performance, e **DEC-055**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Golden suite completa | Runner + fixtures em `tests/golden` cobrindo inventário Ciclo 0 aplicável | Suite CI verde; 0 fail não classificado |
| O-02 | Comparação observável | Diff tipado: exit, stdout, stderr, tree, content, DB/state, HTTP | Cada dimensão com assert ou skip documentado |
| O-03 | Normalizações | Só voláteis §42.2; lista versionada | Review + teste que rejeita over-normalize |
| O-04 | Diferenças classificadas | Cada diff → A/B/C/D + ADR se C | **Zero** diferença sem classificação |
| O-05 | Fuzzing parsers/paths | Harness (proptest e/ou cargo-fuzz) em paths + YAML/JSON críticos | Sem crash / panic em seed fixo CI; corpus mínimo |
| O-06 | Security: injection | Tentativas de shell concat / metachar em spawn | Bloqueadas; SafeCommand/argv only |
| O-07 | Security: env leak | Secrets em env não aparecem em stdout/stderr/logs de erro | Redact + asserts |
| O-08 | Security: archive traversal | zip/tar slip → rejeição tipada | Sem write fora do jail |
| O-09 | Security: signature mismatch | Manifest/asset com sig inválida / skipped | Fail-closed (alinhado 034/053) |
| O-10 | Performance baseline | Startup frio, RSS pico smoke, tamanho `dare` release | Números em `docs/` + gate “não regredir X%” (Blueprint congela limiares) |
| O-11 | Cross-platform | Suite `tests/cross-platform` em Linux + macOS + Windows (CI matrix) | Paths / separators / drive casing OK |
| O-12 | Docs + DEC-055 | `docs/compatibility/parity-hardening.md` (+ security) + DECISION-LOG | Review |
| O-13 | Ralph close | `cargo fmt --check`, clippy `-D warnings`, `cargo test`, `cargo audit` | Exit 0; 0 CVE HIGH/CRITICAL |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Gate de confiança antes de RC / cutover |
| Tech Lead | DARE CLI Rust | DEC-055; classificação A/B/C/D; limiares perf |
| Compat / Baseline | — | Paridade TS 3.18.1; inventário de fixtures |
| Segurança | — | Suite security; CI-010..CI-014 must_fix |
| Operações / Release | CI | Matrix OS; artefato instalável smoke |
| Engenheiro consumidor | — | Comportamento previsível; sem surpresa pós-cutover |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Layout de testes | MUST | Existem `tests/golden`, `tests/security`, `tests/cross-platform` (integration tests workspace ou crate dedicada — Blueprint escolhe) |
| RF-02 | Golden runner | MUST | Comando/target documentado (ex. `cargo test --test golden_*` ou bin `dare-golden`) executa a suite completa em CI |
| RF-03 | Dimensões de comparação | MUST | Cada caso golden declara quais eixos comparar: `exit`, `stdout`, `stderr`, `tree`, `content`, `state`/`DB`, `HTTP` |
| RF-04 | Baseline TS | MUST | Goldens referenciam snapshots ou captura da baseline `@dewtech/dare-cli@3.18.1` **ou** SoT nativo já ADR/DEC-documentado (Classe C) |
| RF-05 | Inventário fixtures | MUST | Cobrir fixtures de `fixtures-inventory.md` aplicáveis a comandos já implementados; gaps listados como skip + issue (não “pass silencioso”) |
| RF-06 | Normalizações permitidas | MUST | Implementar normalizer só para: timestamps, UUIDs/tokens, paths temp, ANSI, separadores de path, drive-letter casing, versão do binário (§42.2) |
| RF-07 | Anti over-normalize | MUST | Teste/regressão: alterar campo de contrato (exit code / flag name / schema key) **falha** o golden mesmo com normalizer ativo |
| RF-08 | Registro de diferenças | MUST | Artefato versionado (ex. `docs/compatibility/parity-diff-log.md` ou seção DEC) com cada diff: id, superfície, classe A/B/C/D, ação, ADR se C |
| RF-09 | Classe D bloqueante | MUST | Qualquer CI-010..CI-014 aberto → suite security FAIL; sem waive em v1 deste microplano |
| RF-10 | Fuzz paths | MUST | Property/fuzz em `SafeRelativePath` / `ProjectRoot` / resolução de paths (proptest MUST; cargo-fuzz SHOULD se CI permitir) |
| RF-11 | Fuzz parsers | MUST | Malformed YAML/JSON (config, DAG, capability-matrix, manifests) não panic; erro tipado |
| RF-12 | Command injection suite | MUST | Fixtures com `;`, `&&`, `$()`, backticks, newlines em args → spawn argv-only; sem shell |
| RF-13 | Env leak suite | MUST | Com `DARE_*` / `GITHUB_TOKEN` / fake secrets no env, outputs de erro/help/golden não ecoam valor |
| RF-14 | Archive traversal suite | MUST | zip/tar com `../` e symlink escape rejeitados em extract paths (update/self/skills/assets conforme aplicável) |
| RF-15 | Signature mismatch suite | MUST | Checksum errado + sig inválida + “signing skipped” cobertos onde o produto verifica (guard / self / skills) |
| RF-16 | Startup measure | MUST | Script/bench mede tempo até `dare --version` / `dare info` (Blueprint fixa comando) em release build |
| RF-17 | Memory measure | MUST | RSS (ou equivalente OS) registrado para smoke curto; metodologia documentada |
| RF-18 | Binary size | MUST | Tamanho do artefato `dare` (e bins auxiliares se houver) registrado; hash opcional |
| RF-19 | Perf gate | SHOULD | Limiares numéricos no Blueprint (ex. startup p95 < N ms; size < M MiB); regressão > X% falha CI ou exige DEC |
| RF-20 | Cross-platform paths | MUST | Casos `windows-path-cases` + separators; CI matrix Linux/macOS/Windows |
| RF-21 | Resolve or document | MUST | Critério de aceite do microplano: **zero** diferença não aprovada — fix B/D ou documentar C com ADR |
| RF-22 | Capability / matrix | SHOULD | Se nova capability (ex. `dare-parity` / `dare-security-suite`) — Blueprint decide; senão só docs |
| RF-23 | Docs | MUST | `docs/compatibility/parity-hardening.md` (golden + normalizações + como rodar) + seção security |
| RF-24 | DEC-055 | MUST | Append-only no `DECISION-LOG.md`; **não** editar DEC-054 |
| RF-25 | Matriz 000A | MUST | Microplano 054 → Concluído ao fechar |
| RF-26 | Release artifact smoke | MUST | Binário release (ou CI artifact) + checksum smoke — alinhado critério “artefato instalável”; **não** exige publicar RC (isso é 055) |
| RF-27 | Mensagens en-US | MUST | Outputs de suite/report em inglês onde forem UX do CLI; docs técnicos PT OK no DARE/ |
| RF-28 | Sem mudança de contrato silenciosa | MUST | Diff Classe A → processo breaking-change / ADR; suite não “ajusta golden” para esconder |

> **MUST** · **SHOULD** · **COULD**

### Superfície de execução (proposta Analyst — Blueprint confirma)

```text
cargo test -p <golden-pkg> --test '*'     # ou cargo test --test golden_*
cargo test -p <sec-pkg> --test security_* # injection, leak, zip-slip, sig
cargo test --test cross_platform_*
# opcional:
cargo fuzz run path_safety                 # SHOULD
scripts/measure-perf.sh | .ps1             # startup / size / rss → docs/perf/baseline-054.md
```

### Princípio de classificação (inegociável)

| Classe | Ação neste microplano |
|--------|------------------------|
| **A** | Preservar; golden deve bater |
| **B** | Corrigir Rust (ou TS legado se ainda relevante) sem ADR |
| **C** | ADR + registro; golden SoT nativo permitido só com `adr_ref` |
| **D** | Must fix segurança; suite vermelha até corrigir |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | Startup frio medido e versionado | Limiar no Blueprint; sem regressão silenciosa |
| RNF-02 | Performance | Tamanho do binário release medido | Limiar no Blueprint |
| RNF-03 | Performance | Memória RSS smoke medida | Metodologia + número baseline |
| RNF-04 | Confiabilidade | Golden/security determinísticos | 0 flake conhecido sem quarantine documentada |
| RNF-05 | Segurança | 0 CVE HIGH/CRITICAL em `cargo audit` | Exit 0 no close |
| RNF-06 | Segurança | Redação de secrets em erros/logs | Suite RF-13 verde |
| RNF-07 | Observabilidade | Relatório de diffs machine-readable (JSON SHOULD) | SchemaVersion 1 no Blueprint |
| RNF-08 | Manutenibilidade | Suites isoladas de unit tests de crates | Falha de golden ≠ “tudo vermelho” opaco |
| RNF-09 | Portabilidade | Linux, macOS, Windows | Matrix CI; skips só com `cfg` + doc |
| RNF-10 | Tempo de CI | Golden+security no PR | Orçamento de tempo documentado; fuzz longo = nightly SHOULD |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Todas as entradas de fixtures/fuzz validadas pelos mesmos gates de produção (paths, limits) | OWASP A03 |
| RS-02 | Secrets/tokens nunca em fixtures commitadas em claro além de placeholders óbvios (`***`, `REDACTED`) | OWASP A02 |
| RS-03 | Suites não elevam privilégio nem escrevem fora de temp/ProjectRoot jail | OWASP A01 / path-safety |
| RS-04 | `cargo audit` sem HIGH/CRITICAL no Ralph close | OWASP A06 |
| RS-05 | Sem secrets hardcoded; tokens de captura baseline só via env CI | Supply chain |
| RS-06 | Path traversal / symlink escape cobertos e bloqueados (CI-010) | Doc Mestre §42.4 |
| RS-07 | Sem shell concatenado; argv separado (CI-011) | SafeCommand |
| RS-08 | Sem leak de env sensível em stdout/stderr/logs (CI-012) | `dare_core::redact` |
| RS-09 | Zip/tar slip rejeitado (CI-013) | Archive extract paths |
| RS-10 | Signature mismatch / skipped fail-closed onde produto assina (CI-014) | guard / self / skills |
| RS-11 | Fuzz não desabilita path jail “para passar” | Anti-bypass |
| RS-12 | Unicode bidi / homoglyphs em paths: pelo menos 1 caso security (SHOULD → MUST se Blueprint achar trivial) | §42.4 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust (workspace) | `rust-version` 1.88 |
| CLI / crates sob teste | `dare-cli` + crates de domínio | workspace |
| Test harness | `assert_cmd`, `predicates`, `tempfile` | pins workspace |
| Property / fuzz | `proptest` MUST; `cargo-fuzz` / libfuzzer SHOULD | `proptest =1.6.0` |
| Hash / sig (fixtures) | `sha2`, padrões já usados em `dare-self` / `dare-guard` | workspace |
| Baseline TS | `@dewtech/dare-cli@3.18.1` (captura offline ou npm CI) | 3.18.1 |
| Docs | `docs/compatibility/*`, DECISION-LOG | — |
| CI | GitHub Actions matrix OS (existente / estender) | — |
| Audit | `cargo audit` | — |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Baseline npm `@dewtech/dare-cli@3.18.1` | Referência de paridade | CLI local / captura | Entrada (fixtures) | exit/stdout/stderr/tree | Compat |
| GitHub Actions | CI | YAML workflows | Execução | Resultados de suite | Release |
| GitHub Releases | Artefato (smoke size) | HTTPS | Entrada opcional | Binário/checksum | Release |
| cargo-audit / RustSec | Advisory DB | HTTPS | Entrada | CVEs | Segurança |
| cosign (se testes sig) | Verify fixture | argv | Local | sig ok/fail | Segurança |

> Sem novas SaaS. Captura TS pode ser pré-gerada e commitada (preferível) para CI hermético.

---

## 9. RESTRIÇÕES

- **Prazo:** Microplano 54/56 — bloqueia 055 (pilotos/RC).
- **Orçamento de infra:** CI matrix 3 OS; fuzz longo só nightly se estourar PR budget.
- **Limitações técnicas:** Sem mudar contratos Classe A sem ADR; sem “corrigir” golden para mascarar bug; paths de teste sob temp + jail.
- **Regulatórias / Compliance:** OWASP alinhado; supply chain (`cargo audit`); sem PII em fixtures.
- **DEC:** Novo id **DEC-055** (054 já usado por self-update).
- **Pré-requisito microplano:** “Todos os comandos planejados implementados” — tratar como **comandos do roadmap até 053**; gaps pós-053 entram como skip classificado, não como escopo de feature nova.

---

## 10. FORA DO ESCOPO (v1)

- Pilotos em projetos reais, shadow paralelo e publicação de **RC** → **055**.
- Cutover npm / descontinuação do legado → **056**.
- Novas features de produto (comandos, MCP tools, self-update extras).
- Mudança de contrato público sem ADR (Classe A).
- Otimizações de performance além de medir + gate de regressão (sem redesign).
- Paridade pixel-perfect de texto humano Classe B cosmético não listado (pode ser B fix ou defer documentado).
- Reescrever toda a suíte unitária existente das crates (reusar; golden/security são a entrega).
- Docker packaging / Scoop (já fora em 053).
- Substituição do TS como SoT onde ADR/DEC já declarou SoT nativo (ex. design/blueprint determinísticos) — apenas registrar na diff log.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Golden flaky (tempo, path, ANSI) | Alta | Alto | Normalizer §42.2 estrito; seeds fixos; sem rede |
| R-02 | Baseline TS indisponível na CI | Média | Alto | Snapshots commitados; sem install npm obrigatório no PR |
| R-03 | Over-normalize esconde Classe A | Média | Alto | RF-07 anti over-normalize; review Tech Lead |
| R-04 | Escopo “todos os comandos” inatingível | Alta | Médio | Inventário + skip explícito; não inventar features |
| R-05 | Fuzz não cabe no PR | Média | Médio | proptest no PR; cargo-fuzz nightly SHOULD |
| R-06 | Falso positivo security em Windows | Média | Médio | cfg + fixtures `windows-path-cases`; matrix real |
| R-07 | Limiares perf arbitrários demais | Média | Médio | Baseline primeiro; gate relativo (%); DEC se mudar |
| R-08 | Confusão DEC-054 vs 055 | Baixa | Médio | Header deste DESIGN; append-only DECISION-LOG |
| R-09 | Suites duplicam e divergem de unit tests | Média | Médio | Golden = observável CLI; unit = domínio; um SoT por eixo |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Escopo golden vs security vs perf alinhado (sem misturar 055/056)
- [ ] Dimensões de comparação (RF-03) aceitas
- [ ] Política de normalização §42.2 aceita + anti over-normalize
- [ ] Classe D (CI-010..014) como bloqueante aceita
- [ ] proptest MUST / cargo-fuzz SHOULD alinhado
- [ ] Limiares de perf: medir agora + gate no Blueprint (números a congelar)
- [ ] DEC id **055** confirmado (054 = self-update)
- [ ] Pré-requisito “todos os comandos” interpretado como até 053 + skips
- [ ] Requisitos de segurança RS-01..12 revisados pelo Tech Lead
- [ ] Fora de escopo alinhado com Product Owner
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-054-hardening-de-paridade-e-seguranca.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | Hardening paridade + security + baselines perf; sem pilotos/RC | 🟢 Microplano 054 · Mestre §42 · §48 |
| ambiguity | Onde moram os testes (`tests/` na raiz vs crate `dare-parity`) | 🟡 proposta: `tests/golden`, `tests/security`, `tests/cross-platform` na raiz do workspace (integration) |
| ambiguity | Captura live TS vs snapshots commitados | 🟡 proposta: snapshots commitados (CI hermético); live opcional local |
| ambiguity | Capability nova vs só docs | 🟡 proposta: **só docs** neste ciclo (sem bump 51→52) salvo Blueprint exigir |
| ambiguity | Limiares numéricos startup/size/RSS | 🔴 Blueprint após primeira medição |
| ambiguity | Cobertura HTTP: só `dare server`/`mcp` smokes já existentes vs suite nova | 🟡 reusar smokes 051/052 + 1 golden HTTP mínimo |
| gap | Pastas `tests/golden|security|cross-platform` ainda não existem | 🟢 criar neste microplano |
| gap | Inventário fixtures vs comandos pós-053 | 🔴 mapear no Blueprint (tabela comando→fixture) |
| gap | Ferramenta única de relatório de diffs | 🟡 Markdown MUST + JSON SHOULD |

### PM

- Aceite v1: golden suite CI verde; security suite cobre injection/leak/zip-slip/sig; cada diff classificada; baselines perf publicadas; DEC-055; Ralph + audit verdes.
- Preferir **não** criar capability só para “ter row” — docs de compatibilidade bastam.
- Congelar limiares de perf no Blueprint **depois** da primeira corrida de medida (evitar inventar números agora).
- Skips explícitos > green falso.

---

## Próximas etapas

1. Revisar e aprovar este Design (especialmente RF-03 dimensões, limiares perf, capability sim/não, interpretação do pré-requisito).
2. Quando aprovado, rodar `/dare-blueprint` com `@DARE/DESIGN-054-hardening-de-paridade-e-seguranca.md`.
3. Em seguida `/dare-tasks` → executar DAG 054.
