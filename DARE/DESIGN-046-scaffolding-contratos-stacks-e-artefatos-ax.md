# DESIGN: Scaffolding — contratos, stacks e artefatos AX (Microplano 046)

> **Versão:** v1.0 | **Data:** 2026-07-25 | **Status:** APPROVED (blueprint autorizado via `/dare-blueprint`)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/046-scaffolding-contratos-stacks-e-artefatos-ax.md`  
> **Referência:** Documento Mestre §12 `dare-scaffold` · §36 Ciclo 18 (`init`/`bootstrap` dependem desta fundação) · skill `dare-ax` · baseline TS `@dewtech/dare-cli@3.18.1` · pré-requisitos **007–010**, **022** · próximo **047**  
> **Posição:** 46 de 56  
> **Arquivo:** `DARE/DESIGN-046-scaffolding-contratos-stacks-e-artefatos-ax.md`  
> **Escopo deste ciclo apenas:** crate **`dare-scaffold`** + **`assets/stacks/**`** + trait **`StackScaffolder`** + registro das **11** stack IDs + modelo backend/frontend/MCP/toolchain/transport + templates + **7** artefatos AX + API **plan / apply / rollback** + validação + fixtures greenfield + docs + **DEC-047**.  
> **Não** CLI `dare init` / `dare bootstrap` (**047**). **Não** hooks/steering (**048**). **Não** self-update (**053**). DEC proposto: **DEC-047**.

---

## 1. DESCRIÇÃO

Construir a **infraestrutura comum de scaffolding** que `dare init` e `dare bootstrap` (047) vão consumir: contratos tipados de stack, templates embutidos, geração dos **sete artefatos AX** e um pipeline **plan → apply → rollback** com path safety — sem ainda expor a UX greenfield no CLI.

O problema: sem crate/templates/contratos estáveis, init/bootstrap viram cópia ad-hoc e quebram paridade com o TS 3.18.1. Quem usa neste ciclo: eng. Rust do CLI e, indiretamente, agentes IDE no ciclo seguinte. Entrega verificável: `crates/dare-scaffold`, `assets/stacks/**`, testes/fixtures, docs de compatibilidade + DEC-047.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Crate `dare-scaffold` no workspace | `cargo test -p dare-scaffold` | Exit 0 |
| O-02 | Trait `StackScaffolder` | API pública estável + ao menos 1 impl por família | Unit |
| O-03 | 11 stack IDs com metadata | Registry fechado; lookup O(1)/map; id desconhecido → InvalidInput | Unit |
| O-04 | Modelo de composição | Types: backend / frontend / MCP / toolchain / transport | Unit + serde roundtrip |
| O-05 | Templates portados | `assets/stacks/<id>/**` (ou embed) para as 11 stacks | Fixture existence |
| O-06 | Sete artefatos AX | Geração + asserts dos 7 paths por stack (lista §4) | Unit × 11 |
| O-07 | Plan determinístico | `ScaffoldPlan` ordenado (path ASC); `--check`/dry-run zero writes | Unit |
| O-08 | Apply + rollback | Falha a meio → rollback restaura estado pré-apply | Integration FS |
| O-09 | Validação pós-apply | Validator por stack (metadata + AX presentes) | Unit |
| O-10 | Fixtures greenfield | Tempdir fixtures para ≥3 stacks representativas (API + MCP + Go/Rust) | Integration |
| O-11 | Docs + DEC-047 | `docs/compatibility/scaffold-contracts.md` + DECISION-LOG; matriz 046 | Review |
| O-12 | Ralph | `fmt`/`clippy`/`test` `-p dare-scaffold` (+ workspace members afetados) | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Desbloquear Ciclo 18 (init/bootstrap) sem adiar AX |
| Tech Lead | DARE CLI Rust | Crate isolada; sem ciclos; DEC-047; paridade 11 stacks |
| Engenheiro | Consumidor (047+) | API clara plan/apply/rollback |
| Compat | Baseline TS 3.18.1 | Diffs Classe A/B/C em IDs/templates |
| Segurança | — | Path jail; sem secrets em templates/AX; rollback em falha parcial |
| Agente / AX | skill `dare-ax` | 7 sinais Discovery/Usage/Defense no greenfield |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Crate `dare-scaffold` | MUST | Member em `Cargo.toml` workspace; deps só `dare-core` / `dare-contracts` / `dare-assets` (sem `dare-cli`) |
| RF-02 | Trait `StackScaffolder` | MUST | Métodos mínimos: `id()`, `metadata()`, `plan(...)`, `validate(...)` (Blueprint congela assinaturas) |
| RF-03 | Registry das 11 stacks | MUST | IDs **exatos** (Mestre §36), case-sensitive: `ruby-rails-8`, `node-nestjs`, `python-fastapi`, `php-laravel`, `rust-axum`, `go-gin`, `go-stdlib`, `mcp-node-ts`, `mcp-python`, `mcp-rust`, `mcp-go` |
| RF-04 | Metadata por stack | MUST | Cada id: kind (`backend`\|`frontend`\|`mcp`\|…), language, default toolchain, default transport (MCP), template root |
| RF-05 | Modelo de composição | MUST | Structs: `BackendStack`, `FrontendStack` (opcional), `McpStack`, `Toolchain` (`none`\|`docker`\|…), `Transport` (`stdio`\|`http`\|`sse` — Blueprint congela enum) |
| RF-06 | `ScaffoldRequest` | MUST | Combina stack(s) + nome projeto + toolchain + transport + flags (`force` reserved for 047) |
| RF-07 | Templates em `assets/stacks/**` | MUST | Árvore por stack id; materialização via `dare-assets` / embed; paths relativos SafeRelativePath |
| RF-08 | Sete artefatos AX | MUST | Gerar (ou garantir) os 7 abaixo em todo scaffold greenfield aplicável (HTTP vs MCP: ver notas) |
| RF-09 | Plan | MUST | `plan(root, req) -> ScaffoldPlan` com items `{ path, action: create\|skip\|replace, kind: template\|ax\|meta }` ordenados path ASC |
| RF-10 | Apply | MUST | `apply(root, plan) -> ScaffoldApplyReport`; cria só sob `ProjectRoot`; journal para rollback |
| RF-11 | Rollback | MUST | Falha mid-apply → restaura criados/overwrite (padrão similar a `dare-update` session journal) |
| RF-12 | Dry-run / check | MUST | Modo zero-write que retorna o mesmo plan/report sem mutar FS |
| RF-13 | Validate outputs | MUST | `validate_stack_output(root, stack_id)` falha se metadata AX incompleta ou template obrigatório ausente |
| RF-14 | Fixtures greenfield | MUST | ≥1 fixture por família: Nest/FastAPI **ou** Rails, Rust/Go, MCP (`mcp-node-ts`) |
| RF-15 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-16 | Exit mapping (lib) | MUST | `InvalidInput` (stack id), `AlreadyExists` (conflito sem force — reserved), path escape → erros core existentes |
| RF-17 | Docs | MUST | `docs/compatibility/scaffold-contracts.md` (IDs, AX list, plan/apply/rollback) |
| RF-18 | DEC-047 | MUST | Append-only em `docs/DECISION-LOG.md` |
| RF-19 | Matriz 046 | MUST | `000A-MATRIZ-DE-STATUS.md` → Concluído no closeout |
| RF-20 | Sem CLI init/bootstrap | MUST | Nenhum `dare init`/`dare bootstrap` neste microplano (só lib + assets + testes) |
| RF-21 | Compat ADR/DEC | SHOULD | Diffs vs TS (ex.: `rails` migrate vs `ruby-rails-8` scaffold) classificados A/B/C |

### 4.1 Sete artefatos AX (proposta congelável — Blueprint confirma paths)

> Fonte: skill `dare-ax` (Discovery / Usage / Defense) + Mestre “sete artefatos AX por stack”. Lista **MUST** no Blueprint; abaixo é a proposta v1:

| # | Artefato | Papel AX |
|---|----------|----------|
| 1 | `llms.txt` | Discovery |
| 2 | `README.md` (secções Bootstrap + Docs) | Discovery |
| 3 | `.env.example` (sem secrets reais) | Usage / Defense |
| 4 | `openapi.json` **ou** `public/openapi.json` | Usage (HTTP); MCP: stub mínimo documentado **ou** `openapi.json` com `paths: {}` + info MCP |
| 5 | `Dockerfile` | Usage |
| 6 | `docker-compose.yml` | Usage |
| 7 | Rate-limit starter (ficheiro idiomático por stack, ex. middleware/config) | Defense |

🟡 Se o inventário TS 3.18.1 divergir, Blueprint substitui a tabela com a lista golden do TS e classifica o diff.

### 4.2 Esboço de API (Blueprint congela)

```rust
pub trait StackScaffolder: Send + Sync {
    fn id(&self) -> &'static str;
    fn metadata(&self) -> &StackMetadata;
    fn plan(&self, root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldPlan>;
    fn validate(&self, root: &ProjectRoot) -> CoreResult<ValidationReport>;
}

pub fn list_stack_ids() -> &'static [&'static str]; // len == 11
pub fn scaffolder_for(id: &str) -> CoreResult<&'static dyn StackScaffolder>;

pub fn plan_scaffold(root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldPlan>;
pub fn apply_scaffold(root: &ProjectRoot, plan: &ScaffoldPlan) -> CoreResult<ScaffoldApplyReport>;
pub fn apply_scaffold_checked(root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldApplyReport>; // plan+apply
// on error mid-apply: rollback then Err
```

### Fora de escopo (ver §10)

- UX `dare init` / `dare bootstrap` e flags `--non-interactive` / `--force` (047)
- Frontend-only stacks (`react`/`vue`) como IDs de primeira classe neste ciclo (podem aparecer só como campo opcional de composição se o Mestre/TS exigir — Blueprint decide)
- Download de pacotes npm/cargo/pip da stack alvo (bootstrap real em 047)
- Hooks on-save / steering (048)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Determinismo | Plan items e JSON reports ordenados (path ASC; ids ASC) | Golden |
| RNF-02 | Performance | Plan das 11 stacks em fixture temp < 2 s CI típico | Smoke timing soft |
| RNF-03 | Portabilidade | Paths `/` normalizados; funciona Win/macOS/Linux | Cross-plat tests |
| RNF-04 | Manutenibilidade | Templates versionados em `assets/stacks`; sem hardcode de conteúdo grande no Rust | Review |
| RNF-05 | Observabilidade | `ScaffoldApplyReport` camelCase: `created[]`, `skipped[]`, `rolledBack` bool | Unit serde |
| RNF-06 | Compat | Paridade observável com TS 3.18.1 nos 11 IDs + 7 AX | Diff table DEC |
| RNF-07 | Isolamento | Sem dependência de `dare-cli` / `dare-graph` / `dare-agent` | `cargo tree -p dare-scaffold` |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar stack id, nome de projeto e paths relativos antes de escrever | OWASP A03 |
| RS-02 | Templates e `.env.example` **sem** senhas/tokens reais; redigir em logs | OWASP A02 |
| RS-03 | Escrita apenas sob `ProjectRoot` via `SafeRelativePath` / path jail | OWASP A01 (confine) |
| RS-04 | `cargo audit` sem CVE HIGH/CRITICAL novas no closeout | OWASP A06 |
| RS-05 | Sem secrets em código/templates commitados; placeholders `${VAR}` / comentários | Supply chain |
| RS-06 | `llms.txt` / README gerados passam scan de padrões `password=`, `api_key=`, `BEGIN PRIVATE KEY` | dare-ax Defense |
| RS-07 | Rollback obrigatório em falha parcial de apply (sem deixar half-written tree “ok”) | Integridade |
| RS-08 | Sem shell concatenado ao materializar; só FS APIs / argv separado se invocar tools | Process safety |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | workspace `rust-version` (1.85+) |
| Crate nova | `dare-scaffold` | `0.1.0-alpha.0` |
| Contratos / erros | `dare-core`, `dare-contracts` | workspace |
| Assets | `dare-assets` + `assets/stacks/**` | workspace |
| Serde | `serde` / `serde_json` | workspace |
| FS tests | `tempfile` | workspace |
| CLI (fora do escopo 046) | `dare-cli` | só no 047 |
| Baseline paridade | `@dewtech/dare-cli` | 3.18.1 |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados | Responsável |
|---------|------|-----------|---------|-------|-------------|
| Filesystem do projeto alvo | Local | FS | Escrita/leitura | Templates + AX | `dare-scaffold` |
| `dare-assets` | Crate interna | API Rust | Leitura | Bytes/templates | Workspace |
| Baseline TS 3.18.1 | Referência | — | Comp. | Golden trees / AX | Compat |
| Registries npm/crates.io/PyPI | — | — | — | **Fora** (047 bootstrap) | — |
| Neo4j / GraphRAG | — | — | — | **Fora** | 040–043 |

---

## 9. RESTRIÇÕES

- Pré-requisitos microplanos **007–010** e **022** considerados satisfeitos na linha Rust atual (matriz pode ainda mostrar ⬜ histórico — validar no Blueprint).
- **Não** implementar comandos CLI neste ciclo.
- IDs de scaffold **≠** necessariamente allowlist de `dare migrate` (`rails` vs `ruby-rails-8`, `react`/`vue` extras) — documentar mapeamento, não unificar à força sem DEC.
- Conteúdo de template por stack pode ser **mínimo viável** (esqueleto + AX), não app completa de produção.
- Orçamento: um DEC (**047**); sem ADR novo salvo quebra de contrato de disco.

---

## 10. FORA DO ESCOPO (v1 deste microplano)

| Item | Motivo |
|------|--------|
| `dare init` / `dare bootstrap` | Microplano **047** |
| Prompts interativos / `--non-interactive` | **047** |
| Instalação de deps da stack (`npm i`, `cargo new` remoto) | Bootstrap real **047** |
| Harness IDE install completo | Já coberto por discover/update; 047 só orquestra |
| Hooks / steering | **048** |
| Dashboard / MCP server do CLI | **051–052** |
| Frontend IDs `react`/`vue` como stacks scaffold de primeira classe | Não estão nas 11 do Mestre §36; opcional composição só se Blueprint exigir |
| `rust-leptos` / `rust-leptos-csr` | Fora das 11 mínimas; ciclo futuro ou extensão pós-047 |

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Lista dos 7 AX divergir do TS golden | Alta | Médio | Congelar no Blueprint contra inventário TS; Classe B/C no DEC |
| R-02 | Templates enormes incham o binário | Média | Médio | Embed seletivo / lazy via `dare-assets`; MVPs por stack |
| R-03 | Conflito de merge com 048/049/050 paralelos em `main.rs` | Baixa* | Médio | *046 não toca CLI*; conflito só se alguém adicionar bin stub cedo |
| R-04 | Rollback incompleto em Win (file locks) | Média | Alto | Journal + testes Win; retry curto; erro explícito se rollback falhar |
| R-05 | Confusão `rails` (migrate) vs `ruby-rails-8` (scaffold) | Alta | Médio | Tabela de alias no docs + InvalidInput com hint |
| R-06 | OpenAPI obrigatório em stacks MCP | Média | Baixo | Stub vazio documentado (RF-08 nota) |
| R-07 | Escopo vazar para 047 (CLI) | Média | Alto | Checklist RF-20; review anti-stub |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] Requisitos funcionais revisados (11 stacks + 7 AX + plan/apply/rollback)
- [ ] Confirmar ou substituir a tabela dos **7 artefatos AX** (§4.1)
- [ ] Confirmar enums `Toolchain` / `Transport` / `Frontend` opcional
- [ ] Segurança (path jail, secrets, rollback) ok
- [ ] Fronteira **046 lib** vs **047 CLI** alinhada com PO
- [ ] Diffs vs migrate allowlist / TS aceitos (DEC-047)
- [ ] Riscos críticos com mitigação
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-046-scaffolding-contratos-stacks-e-artefatos-ax.md`

---

## Próximas etapas

1. Revisar e **aprovar** este Design (ou pedir ajustes — sobretudo §4.1 AX e aliases de stack).
2. Executar `/dare-blueprint` neste ficheiro.
3. `/dare-tasks` → `/dare-dag-run-parallel` com `DARE/dare-dag-046.yaml`.
