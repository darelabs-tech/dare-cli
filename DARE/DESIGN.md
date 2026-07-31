# DESIGN: Governança, baseline e ADRs prioritárias (Microplano 001)

> **Versão:** v1.0 | **Data:** 2026-07-20 | **Status:** DRAFT
>
> **Fonte:** `DARE-RUST-MICRO-PLANOS/001-governanca-baseline-e-adrs-prioritarias.md`
> **Referência:** Documento Mestre — Reescrita do DARE CLI em Rust (`@dewtech/dare-cli` v3.18.1)
> **Posição:** 1 de 56 na sequência de microplanos

---

## 1. DESCRIÇÃO

Este Design cobre a **fase zero de governança** da reescrita nativa do DARE CLI em Rust. O problema que resolve é impedir que a implementação avance sem contratos mensuráveis: hoje a referência TypeScript 3.18.1 existe, mas ainda faltam baseline reproduzível, ADRs prioritárias e regras explícitas para classificar incompatibilidade, idioma, JSON, versionamento de disco e breaking changes.

O entregável é documental e processável — ADRs em `docs/adr`, políticas em `docs/compatibility`, registro de decisões/responsáveis e baseline com hash da versão TypeScript — para que engenheiros e agentes saibam o que preservar, o que corrigir e o que exige aprovação antes de mudar comportamento. Usuários finais (devs que usam `dare`) não interagem com este ciclo; o público são o time DARE Labs, revisores de ADR e quem vai executar os microplanos 002+.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Congelar baseline TypeScript reproduzível | Arquivo de baseline com versão + hash (ex.: SHA-256 do tarball/commit/`package` de referência) | `3.18.1` + hash único documentado e verificável |
| O-02 | Aprovar ADRs prioritárias do ciclo 0 | Contagem de ADRs 001/002/004/006/007 com status `Accepted` | 5/5 aprovadas |
| O-03 | Eliminar ambiguidade de compatibilidade | 100% das diferenças conhecidas classificadas (A/B/C/D) em `docs/compatibility` | 0 diferenças sem classe |
| O-04 | Institucionalizar breaking changes | Processo documentado + registro de decisões com responsável nomeado | 1 processo + ≥1 registro inicial |
| O-05 | Habilitar microplano 002 sem bloqueio de governança | Checklist de aceite do microplano 001 | 100% dos critérios MUST marcados |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Escopo da reescrita, canais alpha→stable, cutover do legado TS |
| Tech Lead | Time DARE CLI Rust | ADRs, contratos, veto a breaking change sem processo |
| Engenheiro de plataforma / CLI | Time implementação | Baseline, fixtures golden, ordem dos microplanos |
| Usuário Final (indireto) | Devs que usam `@dewtech/dare-cli` | Paridade observável e migração previsível (não consomem este ciclo) |
| Operações / Release | Quem publica GitHub Releases | Changelog, matriz de compatibilidade, SBOM/checksums nos ciclos seguintes |
| Segurança | Tech Lead + revisão OWASP | Classe D (vulnerabilidades) sempre corrigidas; secrets fora de docs/código |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Registrar versão TypeScript de referência e hash | MUST | Documento em `docs/compatibility` (ou equivalente) cita `@dewtech/dare-cli@3.18.1`, origem (npm/git), e hash verificável; comando de verificação documentado |
| RF-02 | Criar ADR-001 — Compatibilidade de bugs legados | MUST | Arquivo em `docs/adr/` com classes A/B/C/D, exemplos do Doc Mestre §44, status Accepted |
| RF-03 | Criar ADR-002 — Contrato de saída JSON | MUST | ADR define estabilidade de schema/campos, versionamento ou estabilidade, e o que é breaking em `--json` |
| RF-04 | Criar ADR-004 — REST compatível e MCP real | MUST | ADR deixa explícito que REST e MCP são transportes distintos (sem substituição silenciosa), alinhado ao Doc Mestre |
| RF-05 | Criar ADR-006 — Compatibilidade e migração do Graph DB | MUST | ADR cobre `.dare/graph.db` / `.dare/graph.json`, BLOB f32 LE e regra de não migrar silenciosamente |
| RF-06 | Criar ADR-007 — Formato canônico de capabilities | MUST | ADR distingue skills-pacote vs capabilities de IDE e aponta modelo canônico |
| RF-07 | Classificar contratos públicos, bugs cosméticos, comportamentais e vulnerabilidades | MUST | Tabela/matriz em `docs/compatibility` mapeando itens conhecidos às classes A–D; sem item “não classificado” |
| RF-08 | Definir política de idioma da CLI | MUST | Política escrita (PT/EN/misto → alvo); referência a ADR-003 futura se o texto completo for adiado, mas a regra operacional do ciclo 0 fica explícita |
| RF-09 | Definir política de JSON, versionamento de disco e compatibilidade | MUST | Documento cobre writers canônicos, campos opcionais vs breaking, leitura legada obrigatória e alteração de exit code como breaking (Doc Mestre §13.3) |
| RF-10 | Criar registro de decisões e responsáveis | MUST | `docs/` (ex.: `DECISION-LOG.md` ou seção no índice ADR) com data, decisão, ADR vinculado e responsável |
| RF-11 | Definir processo de aprovação de breaking changes | MUST | Fluxo: proposta → ADR → revisão Tech Lead/PO → changelog + migration note → merge; exit codes/flags/schemas listados como breaking |
| RF-12 | Inventariar fixtures/golden relevantes à governança | SHOULD | Lista mínima de fixtures a preservar (vazias/legado/invalid-config) referenciada na baseline |
| RF-13 | Issue principal + subtarefas rastreáveis | SHOULD | Issue/épico do microplano 001 com subtarefas espelhando RF-01–RF-11 |

> Prioridades: **MUST** (bloqueia v1 deste microplano) · **SHOULD** (importante, mas não bloqueia sozinho) · **COULD** (nice to have)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Manutenibilidade | ADRs e políticas legíveis por engenheiro novo em ≤ 30 min | Índice + 5 ADRs + 1 doc de compatibilidade |
| RNF-02 | Determinismo | Baseline e hashes reproduzíveis em Linux, macOS e Windows | Mesmo hash para o mesmo artefato de referência |
| RNF-03 | Observabilidade | Decisões e incompatibilidades rastreáveis | Toda diferença intencional aponta ADR + changelog |
| RNF-04 | Qualidade de build | Se existir workspace Cargo neste repo ao fechar o microplano: `cargo fmt --check`, `cargo clippy`, `cargo test` | Exit 0 nos três; se workspace ainda não existir, documentar deferência explícita ao microplano 002 no decision log (não deixar critério “órfão”) |
| RNF-05 | Release / CI | Artefato instalável ou job CI que prove a baseline (mesmo que só docs + stub) | Pelo menos um artefato ou workflow verde associado ao ciclo 0 |
| RNF-06 | Segurança documental | Nenhum secret, token ou PII em ADRs, baseline ou logs de exemplo | Revisão manual no checklist |
| RNF-07 | Cross-platform | Políticas de path e ordenação independentes de locale já enunciadas na governança | Paths internos `/`; ordenação determinística citada nas políticas |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Entradas futuras (paths, configs, JSON/YAML) devem ser validadas antes de processar; esta fase já documenta a obrigação nos ADRs/políticas | OWASP A03 |
| RS-02 | Nenhum segredo ou credencial em texto plano em docs, fixtures de exemplo ou decision log; hashes são de artefatos públicos, não de secrets | OWASP A02 |
| RS-03 | Breaking changes e alterações de contrato só com aprovação nomeada (Tech Lead/PO); sem “atalho” de merge | OWASP A01 (governança de mudança) |
| RS-04 | Dependências auditadas antes de cada release futuro; política Classe D obriga corrigir vulnerabilidades mesmo com mudança de comportamento | OWASP A06 |
| RS-05 | Secrets via env/vault — ADRs e exemplos nunca embutem tokens | Supply chain |
| RS-06 | Path safety, argv separado (sem shell concatenado) e redação de secrets em logs/erros ficam como invariantes de segurança no ADR-001 (Classe D) e nas políticas de compatibilidade | Segurança por padrão — Doc Mestre §11.5 |
| RS-07 | Classe D (path escape, execução insegura, secret leakage, assinatura inválida, zip-slip) = correção obrigatória, sem optar por “paridade com o bug” | Política de bugs §44 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem alvo da reescrita | Rust | [A definir no microplano 002 — toolchain pinada] |
| Baseline de referência | `@dewtech/dare-cli` (TypeScript) | **3.18.1** (hash a registrar) |
| Documentação de decisões | Markdown ADR | `docs/adr/` |
| Políticas de compatibilidade | Markdown | `docs/compatibility/` |
| Controle de versão | Git | repositório atual `dare-cli` |
| Issue tracking | [A definir] (GitHub Issues / Linear / etc.) | Rastreio do épico 001 |
| CI / qualidade (gate) | cargo fmt / clippy / test (+ CI cross-platform no 003) | Após workspace 002, ou stub mínimo se exigido no fechamento de 001 |
| Distribuição (ciclos seguintes) | GitHub Releases | native-first; npm só ponte legada |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| npm registry / pacote `@dewtech/dare-cli` | Baseline de referência | HTTPS / tarball | Entrada (leitura) | Versão 3.18.1, artefato para hash | Time DARE CLI |
| Repositório TypeScript de referência | Código fonte | Git | Entrada (leitura) | Fixtures, golden outputs, comportamento observável | Time DARE CLI |
| GitHub Issues (ou tracker escolhido) | Gestão | HTTPS | Saída + entrada | Épico/subtarefas do microplano 001 | Tech Lead |
| Claude / Cursor / Codex / Antigravity | Harnesses (contexto futuro) | N/A neste ciclo | — | Apenas mencionados em ADR-007; sem integração operacional no 001 | — |

> Nenhuma integração de runtime de produto neste microplano — só leitura da baseline e publicação de docs.

---

## 9. RESTRIÇÕES

- **Prazo:** Microplano 001 é pré-requisito duro do 002 (workspace Rust); não avançar a implementação de domínio sem ADRs Accepted.
- **Orçamento de infra:** Sem custo de infra além de storage Git/CI existente; sem serviços pagos novos neste ciclo.
- **Limitações técnicas:** Escopo limitado a `docs/adr` e `docs/compatibility` (+ decision log). Sem mudar contratos de disco/código de produto sem ADR aprovado. Sem otimizações prematuras. Documento mestre deve estar aprovado (pré-requisito do microplano).
- **Regulatórias / Compliance:** Sem PCI/HIPAA neste ciclo; LGPD aplicável apenas no sentido de não registrar PII em exemplos/logs. Telemetria/opt-in fica para ADR-011 (fora deste conjunto prioritário).
- **Idioma dos artefatos deste ciclo:** documentação de governança em **português** (alinhado ao time); política de idioma da CLI (mensagens runtime) é entregável RF-08 e pode divergir do idioma dos ADRs.

---

## 10. FORA DO ESCOPO (v1)

- Funcionalidades dos microplanos 002–056 (workspace, CI, comandos, DAG, GraphRAG, etc.).
- Redigir ADR-003, ADR-005, ADR-008–ADR-012 na íntegra (exceto política operacional mínima de idioma/disco citada em RF-08/RF-09 que pode apontar para ADR futuro).
- Implementar código de domínio Rust, parsers, ou golden runner completo (apenas inventário/referência).
- Mudanças de contrato público sem ADR aprovado.
- Otimizações de performance sem benchmark.
- Cutover da versão TypeScript / descontinuação do npm.
- Assinatura de releases, auto-update, telemetria e skill registry protocol (ADRs posteriores).
- Scaffold greenfield (`dare init` / `bootstrap`) — decisão já tomada: discover-first.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | ADRs genéricas demais → ambiguidade na implementação | Média | Alto | Critérios de aceite por ADR com exemplos concretos do Doc Mestre §13.3 e §44 |
| R-02 | Hash/baseline não reproduzível entre máquinas | Média | Alto | Documentar fonte exata (npm tarball vs commit) e comando de verificação cross-OS |
| R-03 | Critérios `cargo *` / release conflitam com ausência de workspace (002) | Alta | Médio | Decision log: stub mínimo **ou** waiver explícito “gate de cargo transferido ao 002”; não fechar 001 com critério silencioso |
| R-04 | Pressão para “corrigir tudo” sem classificar (quebra paridade CI) | Média | Alto | Classe A preservada; B corrige; C exige ADR; D obriga fix — checklist no PR |
| R-05 | Idioma misto da CLI gera discussão interminável | Média | Médio | RF-08 decide regra operacional agora; ADR-003 formaliza depois se necessário |
| R-06 | Documento mestre ainda não formalmente aprovado | Baixa | Alto | Bloquear merge dos ADRs Accepted até PO confirmar Doc Mestre |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Requisitos funcionais revisados e priorizados (RF-01–RF-13)
- [ ] Requisitos de segurança validados pelo Tech Lead (RS-01–RS-07)
- [ ] Stack técnica / escopo documental aprovados (`docs/adr`, `docs/compatibility`)
- [ ] Baseline 3.18.1 + estratégia de hash confirmadas
- [ ] Lista de ADRs prioritárias (001, 002, 004, 006, 007) confirmada — sem expandir para 003/005/008–012 neste ciclo
- [ ] Integrações externas confirmadas (só leitura da baseline TS + tracker)
- [ ] Fora do escopo alinhado com Product Owner
- [ ] Riscos críticos com mitigação definida (especialmente R-03 cargo vs 002)
- [ ] Pré-requisito “Documento mestre aprovado” confirmado
- [ ] Pronto para `/dare-blueprint` deste Design

---

## Apêndice A — Mapa rápido das classes de incompatibilidade (a formalizar no ADR-001)

| Classe | Nome | Ação |
|--------|------|------|
| A | Contrato público | Preservar salvo breaking aprovada (exit codes, flags, schemas, IDs, comportamento de CI) |
| B | Bug cosmético | Corrigir e documentar (ex.: `dare new` no welcome, mojibake) |
| C | Bug comportamental potencialmente utilizado | ADR + migration note |
| D | Vulnerabilidade | Corrigir obrigatoriamente |

## Apêndice B — ADRs deste microplano vs backlog

| ADR | Título | Neste microplano |
|-----|--------|------------------|
| ADR-001 | Compatibilidade de bugs legados | **Sim** |
| ADR-002 | Contrato de saída JSON | **Sim** |
| ADR-003 | Idioma da CLI | Política mínima (RF-08); ADR completo opcional/adiado |
| ADR-004 | REST compatível e MCP real | **Sim** |
| ADR-005 | Protocolo e formato do skill registry | Não |
| ADR-006 | Compatibilidade e migração do Graph DB | **Sim** |
| ADR-007 | Formato canônico de capabilities | **Sim** |
| ADR-008–012 | Assinatura, auto-update, Claude API, telemetria, versionamento disco | Política mínima de disco (RF-09); ADRs completos depois |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design (checkboxes da §12).
2. Executar `/dare-blueprint` com base em `DARE/DESIGN.md`.
3. Implementar entregáveis do microplano 001; só então avançar para `002-workspace-rust-e-toolchain.md`.
