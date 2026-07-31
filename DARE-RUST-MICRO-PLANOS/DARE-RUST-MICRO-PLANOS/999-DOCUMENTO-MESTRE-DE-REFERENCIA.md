# Documento Mestre — Reescrita do DARE CLI em Rust

> **Status:** baseline técnica e plano de implementação
>
> **Versão de referência:** `@dewtech/dare-cli` v3.18.1
>
> **Data de consolidação:** 2026-07-20
>
> **Objetivo:** reunir em um único documento o mapeamento da implementação TypeScript atual e o plano incremental para uma nova implementação nativa em Rust.

---

## 0. Resumo executivo

O DARE CLI será **reescrito do zero em Rust**, usando a versão TypeScript 3.18.1 como implementação de referência. Não existe implementação Rust anterior a ser portada. O trabalho será conduzido como uma substituição incremental por **fatias verticais**, com um CLI nativo utilizável desde o primeiro ciclo.

A estratégia possui cinco compromissos centrais:

1. **Paridade explícita:** comandos, flags, códigos de saída, formatos de arquivos, estado persistido e artefatos de IDE serão tratados como contratos verificáveis.
2. **Entrega incremental:** cada ciclo publicará um binário nativo funcional, adicionando comandos e capabilities de forma cumulativa.
3. **Suporte aos quatro harnesses:** Claude Code, Cursor, Codex e Antigravity serão suportados desde o primeiro ciclo que instala o DARE em um projeto.
4. **Distribuição native-first:** GitHub Releases será a origem oficial dos binários; npm e Node.js não serão requisitos da implementação Rust.
5. **Correção consciente:** bugs acidentais da versão TypeScript não serão reproduzidos automaticamente. Cada incompatibilidade será classificada e decidida por ADR.

### 0.1 Resultado esperado

Ao final da iniciativa, o DARE será composto por:

```text
Código-fonte Rust
        │
        ├── binário `dare`
        ├── binário/serviço `dare-server` ou `dare-mcp-server`
        ├── assets canônicos incorporados
        ├── suporte a Claude, Cursor, Codex e Antigravity
        ├── DAG, GraphRAG, verificação, guard e agentes
        └── distribuição nativa para Linux, macOS e Windows
```

### 0.2 Decisões já tomadas

- A nova implementação será nativa em Rust.
- O npm poderá existir apenas como ponte temporária para usuários legados.
- O primeiro ciclo útil será orientado a projetos existentes com `dare discover`, não a scaffold greenfield com `dare init`.
- `dare init` e `dare bootstrap` serão implementados quando a infraestrutura de stacks, templates e contratos já estiver madura.
- Skills do registry e artefatos das IDEs serão modelados como camadas distintas.
- O `execute` será migrado em vários ciclos, porque concentra DAG, verificação, agentes, worktrees, orçamento e replanejamento.
- REST compatível e MCP real serão tratados como transportes distintos, não como substituições silenciosas.

### 0.3 Critério de sucesso global

A versão Rust só substituirá a TypeScript quando:

- os 25 comandos top-level e seus subcomandos necessários estiverem cobertos;
- flags públicas e códigos de saída estiverem documentados e testados;
- contratos de disco tiverem compatibilidade ou migração explícita;
- Claude, Cursor, Codex e Antigravity passarem na matriz de capabilities;
- os scaffolders e os sete artefatos AX por stack estiverem cobertos;
- GraphRAG, DAG, guard, verificação, dashboard e servidor estiverem operacionais;
- Linux, macOS e Windows forem suportados por releases assinados;
- a execução não depender de Node.js ou `node_modules`;
- os testes golden TypeScript × Rust atingirem os critérios de paridade definidos neste documento.

---

## Parte I — Baseline técnica da versão TypeScript

Esta parte documenta o comportamento que deve ser preservado, migrado ou deliberadamente alterado. A fonte analisada foi `versao-typescript/dare-method/packages/cli`, com aproximadamente 235 arquivos TypeScript e 42 mil linhas, além do pacote de produção `@dewtech/dare-cli` v3.18.1.

### 1. Visão geral

- **Pacote:** `@dewtech/dare-cli` v3.18.1 — monorepo pnpm, pacote único.
- **Binários:** `dare` (CLI principal) e `dare-mcp-server` (servidor HTTP de contexto).
- **Filosofia:** o CLI é **orquestrador determinístico**; o agente da IDE (Claude Code, Cursor, Antigravity, Codex) executa as tasks. O CLI só chama LLM diretamente em 1 ponto (driver `claude` do `dare execute --agent`).
- **Metodologia DARE:** Design → Architecture (Blueprint) → Review → Execute, com fases brownfield (reverse/dna/migrate/patterns/discover).

### 2. Superfície de comandos (25 comandos top-level)

Entrypoint: `src/bin/dare.ts` (commander). Flag global `--no-banner`; banner figlet+gradient só em `init`, `--version`, `welcome` (suprimido se não-TTY, `DARE_NO_BANNER=1`).

**Flags AI compartilhadas** (`--ai`, `--provider <codex|claude-code|cursor-cli|antigravity-cli|mock>`, `--json`) em: `design, blueprint, reverse, dna, migrate, patterns, review, refine`.

| Comando | Função | Flags principais |
|---|---|---|
| `init [nome]` | Scaffold interativo de projeto novo (inquirer) | `--stack`, `--fullstack`, `--mcp <lang>`, `--transport`, `--toolchain`, `--non-interactive` |
| `bootstrap` | Roda scaffolder oficial da stack do `dare.config.json` | `--force`, `--toolchain` |
| `discover` | Detecta projeto brownfield e instala DARE | `-d`, `--check` |
| `reverse` | Engenharia reversa Fase 0 → IDEIA.md + specs | `-d`, `--check`, `--modules`, `--no-excalidraw`, `--report`, `--deep`, `--ast` + AI |
| `dna` | Extrai convenções → DARE/PROJECT-DNA.md | `-d`, `--check`, `--ast` + AI |
| `migrate` | Plano de migração + Gherkin de paridade | `-d`, `--to <stack>`, `--check` + AI |
| `design <desc>` | Gera DARE/DESIGN.md | `--interactive` + AI |
| `blueprint [design]` | Scaffolda BLUEPRINT.md, TASKS.md, dare-dag.yaml, EXECUTION/ | `-f, --force` + AI |
| `execute` | **Coração do orquestrador** (ver §2.1) | ~25 flags |
| `graph <sub>` | Knowledge graph: `stats, query, viz, owners, impact, trace, locate, drift, ingest` | por subcomando |
| `dag viz` | Renderiza DAG estático | `--dag`, `-f mermaid\|dot\|excalidraw`, `-o` |
| `validate` | Valida dare-dag.yaml (ciclos, ids, refs) | `--dag`, `--strict` |
| `info` | Read-only: versões, artefatos, progresso | — |
| `update` | Sincroniza artefatos de IDE com a versão do CLI | `--dry-run`, `-y`, `--force`, `--target` |
| `review <task-id>` | Anti-stub/mock/TODO estático + verdito semântico | `--strict`, `--errors-only`, `--files`, `--from-agent`, `--format`, `--comment`, `--fail-on` + AI |
| `refine <task-id>` | Mede complexidade, propõe/aplica split | `--split`, `--apply`, `--strict`, `--format`, `--from-agent` + AI |
| `bench` | Harness de fixtures (Fix·Rate, solve-rate) | `--suite`, `--json`, `--baseline`, `--fail-on-regression`, `--filter` |
| `steering <sub>` | `list`, `show <file>` — steering files | `--json` |
| `hooks <sub>` | `list`, `run <evento>`, `validate` — trust gate RS-05 | `--file`, `--task`, `--trust`, `--json` |
| `patterns` | Mineração determinística → DARE/PATTERNS.md | `-d`, `--check`, `--modules`, `--inject`, `--ast` + AI |
| `guard [target]` | Scan unicode + prompt-injection + proveniência | `--staged`, `--all`, `--strict`, `--format`, `--comment`, `--fail-on`, `--sign`, `--unicode strip\|block` |
| `dashboard` | Dashboard local de telemetria (Express, porta 4100) | `--port`, `--no-open` |
| `skill <sub>` | `list, info, add, remove, update, publish` (ver §4) | por subcomando |
| `ai <sub>` | `doctor, providers, run, prompt` — providers terminal-first | por subcomando |
| `welcome` | Banner + quick-start (menciona `dare new`, que **não existe** — bug) | — |

#### 2.1 `dare execute` em detalhe

Ações mutuamente exclusivas: `--status` (default), `--next`, `--watch`, `--complete <id>`, `--fail <id>`, `--reset <id>`, `--agent`.

- **`--next`:** cascading-skip → `nextExecutableTasks` (menor rank executável) → prompt composto (`subtask_prompt` + graph-locate opcional + tail dos outputs dos pais capado em `parent_context_chars`).
- **`--complete <id>`:** Ralph Loop obrigatório (build→test→lint por stack) → gate `dare review` opcional (`review.onComplete`) → verificação pós-Ralph (fail-to-pass, anti-tamper, mutation, formal; best-of-N com seleção Pareto) → telemetria no grafo → `markDone` + indexação semântica incremental. Falha em qualquer gate bloqueia DONE.
- **`--agent`:** loop autônomo com driver (`claude|codex|cursor|antigravity|mock`), guard preflight (FAIL → exit 6), N candidatos em worktrees `.dare/agent-worktrees/`, `BudgetTracker` (`--budget-tokens`), política de decay (`DONE|CONTINUE|FRESH_START|REPLAN|ESCALATE|STOP`), REPLAN via split + `spliceSubDag`, aprovação por rank (`--require-approval rank|none`, exige TTY).
- Flags de verificação: `--verify/--no-verify`, `--full-mutation`, `--verdict-json`, `--best-of <n>`, `--policy decay|fixed`, `--prerank`, `--formal/--no-formal`, `--formal-backend dafny|verus|lean`.

#### 2.2 Exit codes especiais (preservar no port)

| Código | Significado |
|---|---|
| 0 | sucesso |
| 1 | erro genérico |
| 2 | bench (suite/baseline inválido), refine `--strict` HIGH/CRITICAL, hooks (config/trust/evento inválido) |
| 6 | `GUARD_FAIL_EXIT_CODE` (guard FAIL; preflight do execute --agent) |
| 7 | `graph drift --strict` com threshold estourado |
| 124 | timeout de processo externo (safe-spawn / ralph) |

### 3. Suporte a IDEs / agentes de código

6 valores de `ide` em `dare.config.json`: `claude-code`, `codex`, `cursor`, `antigravity`, `hybrid` (cursor+antigravity), `claude-hybrid` (claude+cursor).

Instalação: `init`/`discover` → `installIdeFiles()` (`src/utils/project-generator.ts`); brownfield (`reverse`/`dna`/`migrate`) → `ensureDareSkills()` — sem config instala para **todas** as IDEs.

| IDE | Arquivo raiz | Diretórios instalados | Conteúdo |
|---|---|---|---|
| **Claude Code** | `CLAUDE.md` (template + fallback dinâmico) | `.claude/commands/` (49 arquivos `.md`), `.claude/settings.json` (**gerado dinamicamente**: permissions + hook PostToolUse → `dare hooks run on-save`) | Commands em markdown puro, sem frontmatter. Não usa `.claude/skills/` |
| **Cursor** | `.cursorrules` (sempre dinâmico) | `.cursor/commands/` (33 `.md`), `.cursor/rules/` (25 `.mdc` com frontmatter `description/globs/alwaysApply`) | `skill-laravel-api.mdc` condicional ao backend; skills de stack geradas dinamicamente |
| **Antigravity** | `.antigravityrules` (dinâmico) | `.agents/skills/<nome>/SKILL.md` (48 skills, frontmatter `name`+`description`), `.agents/workflows/` (vazio) | Formato Agent Skills |
| **Codex** | `AGENTS.md` (**sempre dinâmico**, `generateCodexRules`) | **reusa** `.agents/skills/` do Antigravity (invocáveis com `$skill-name`) | Sem pasta `.codex/`; detecção por `.codex/` ou `AGENTS.md` existente |

- Slash commands do Claude (49): todos os `dare-*` (fases, brownfield, grafo, infra) + 5 `skill-*` de stack. Cursor tem 33 commands + 25 rules (docker dividido em `dare-dockerfile`/`dare-docker-compose`; ax/dag-build/dag-runner viram rules). Antigravity tem 48 skills (sem `dare-dag-viz`).
- `templates/ide/cursor/templates/` é a **fonte canônica** dos 6 templates DARE (BLUEPRINT, DESIGN, TASK-SPEC, TASKS, TELEMETRY, HOOKS-ADAPTER) copiados para `<projeto>/templates/` em todas as IDEs.
- Sempre instalados: `dare.config.json`, `DARE/` (README + EXECUTION/), `templates/`, `dare-graph.yml`, merge de `.gitignore`.
- **`dare update`:** guiado por `templates/UPDATE-MANIFEST.json` (schemaVersion 1; releases 2.16.0→3.8.2 documentadas — 3.9+ sem entradas). Detecção de customização por SHA-256 (`identical|missing|apply|customized`), políticas keep/replace/ask, backup em `.dare/backup-<versão>/`, migrations de schema (paths `dare.config.json#bloco`). **Codex não aparece em nenhum `appliesTo`** — só recebe changes `*`.
- Driver Codex de execução: `codex exec --json --sandbox <mode> --ask-for-approval <mode>` com parse de eventos JSONL; env `DARE_CODEX_COMMAND`. GEMINI.md não existe.

### 4. Sistema de skills (duas camadas distintas)

#### 4.1 Skills-pacote (`dare skill`)
- Pacotes TypeScript com `skill.yml` (obrigatórios: `name, version, description, author, license, dare_version`; **license deve ser MIT** — regra D-001).
- Manifest do projeto: `.dare/skills.yml` (`skills: [{name, version, enabled, dependsOn?}]`), sem lockfile/hash.
- Registries (prioridade remote > local > mock):
  - **Remote:** `https://dare-registry.vercel.app` (`GET /api/skills`, `GET /api/skills/<name>`, `POST /api/publish/<name>` com Bearer GitHub token; timeout 3 s, nunca lança).
  - **Local:** `~/.dare/registry/<name>/<versão>/` + `index.json` (override `DARE_LOCAL_REGISTRY`).
  - **Mock:** `registry-mock.json` embarcado (7 skills) — **única fonte** usada por `info` e resolução de dependências.
- Instalação: copia para `<projeto>/packages/skills/<name>/`; deps resolvidas topologicamente (todas dependem de `dare-ax`).
- 6 skills embutidas: `dare-ax` (AX/llms.txt), `dare-frontend-design`, `dare-layered-design`, `dare-llm-integration`, `dare-quality-telemetry`, `dare-realtime`.
- **Inconsistências conhecidas** (decidir se preserva ou corrige no port): `skill update` só atualiza manifest (não recopia arquivos); `remove` não apaga arquivos; publish remoto envia só metadados (sem tarball); 5/6 `skill.yml` embutidos sem `dare_version` (falhariam no próprio publish).

#### 4.2 Skills/comandos de IDE
Arquivos markdown copiados de `templates/ide/**` (ver §3). Fonte de verdade declarada: `implementations/<ide>/` sincronizado no build (`scripts/sync-implementations.ts`).

### 5. Engines core

#### 5.1 GraphRAG (`src/graphrag/`)
- Interface `KnowledgeGraph`, 3 backends via `dare-graph.yml`: **sqlite** (sql.js WASM, default, `.dare/graph.db` — reescreve o arquivo inteiro a cada mutação), **json** (`.dare/graph.json`), **neo4j** (HTTP, experimental).
- Schema SQL: tabelas `nodes(id, type, label, description, vector BLOB, metadata, created_at, updated_at)` e `edges(id, source_id, target_id, type, weight, metadata)` + 4 índices. `vector` = Float32Array little-endian.
- 12 NodeTypes (`task, file, schema, endpoint, component, entity, concept, gate, code_symbol, requirement, pattern, formal-gate`), 13 EdgeTypes. IDs canônicos: `task:{id}`, `file:{posixPath}`, `code_symbol:{path}::{symbol}`, `edge {kind}:{from}->{to}`.
- **Busca híbrida:** keyword (LIKE) + vetorial (cosine O(n·d), sem ANN) + grafo (BFS 2 hops), fundidas por **RRF** (`1/(60+rank)`). Embeddings: `@huggingface/transformers` opcional, modelo `all-MiniLM-L6-v2` (384 dims), fallback silencioso keyword-only.
- Traverse: BFS caps maxHops≤5/fanout≤200, ordenação determinística. Locate: seeds qualifiedName/path/keyword com scores decaindo −0,15/hop.
- Indexação incremental por `contentHash` sha256; code-index por **regex** (não AST); drift (`orphan-requirement`, `orphan-code`, `stale`).

#### 5.2 DAG Runner (`src/dag-runner/`)
- `dare-dag.yaml` v2.1: `title, version, limits{parent_context_chars:2000, task_output_chars:4000, timeout_seconds:600}, models{runner→{HIGH,MED,LOW}}, tasks[{id, title, depends_on, complexity, subtask_prompt, spec_file}]` (+ schema legado flat).
- Estado runtime **separado** em `.dare/state.json` v1: `{version:1, updatedAt, tasks{id: {status, output, error, tokens, duration, attempts[{n, at, passed, failureSignature, failedAspect}], parentId, dependsOn}}}`.
- Ranks: longest-path DFS memoizado (equivalente topológico a Kahn) com detecção de ciclo; cadência rank-a-rank; cascading-skip por fixpoint.
- Canvas `DARE/.canvas.md` (tabela + barra de progresso). Ingestão pós-DONE no grafo por heurísticas regex do output.
- **Ralph Loop:** gates build→test→lint por stack (tabela hardcoded laravel/nestjs/fastapi/axum/go/react/vue/leptos/mcp-*), timeout 600 s (exit 124).
- Sub-DAG (REPLAN): splice com `CycleError`/`MaxDepthError` (depth 2), persistido só no state via `__parentId`.

#### 5.3 AST (`src/ast/`)
- web-tree-sitter 0.25.10 + tree-sitter-wasms (opcionais; fallback **regex** transparente). Gramáticas: typescript, tsx, javascript, python, php, go, ruby, rust.
- Extrai endpoints HTTP + entities (classes/models) → `DataModel`; merge AST×regex com dedupe. Usado por `reverse`, `dna`, `patterns` (o GraphRAG não usa tree-sitter).
- Em Rust: tree-sitter **nativo** + crates de gramática — elimina WASM, um dos maiores ganhos de performance.

#### 5.4 Guard (`src/guard/`)
Pipeline de 3 camadas → verdict `PASS|WARN|FAIL`:
1. **Unicode:** zero-width, bidi override, variation selectors, tag chars, homoglyphs; modo `strip` (sanitiza) ou `block`.
2. **Scan:** regras regex de prompt-injection em `rules/scan-rules.json` (4 built-in: instr-override, shell-exec, exfiltration, hidden-directive; override `DARE_GUARD_SCAN_RULES_PATH`); evidência redigida.
3. **Proveniência:** classificação human/agent/external por `trustedPaths`; artefatos "control" exigem assinatura **minisign/Ed25519** (`<file>.minisig`) se `signing.enabled`; `dare guard --sign` assina com `DARE_GUARD_PRIVATE_KEY`.

#### 5.5 Verificação (`src/verification/`, `src/exec/`)
- `safe-spawn`: sempre argv (`shell:false`), env com allowlist (**remove** `SECRET|TOKEN|KEY|PASSWORD|...`), truncagem 4000 chars, timeout SIGTERM → 124.
- Aspectos: `build|test|lint|type|fail-to-pass|anti-tamper|mutation|formal`. Baseline em `.dare/verification/<taskId>.json`.
- Mutation testing por adapter: stryker (node), mutmut (python), cargo-mutants (rust), infection (php); score ≥0,70, incremental sobre git diff.
- Formal: dafny (default) | verus | lean; alvos por tag `@dare-formal`; anti-bypass obrigatório; repair loop ≤5.
- Decay: `failureSignature` = sha256[0..8] de aspecto+stderr normalizado; saturação em janela de 3 → fresh-start/replan/escalate; máx 5 tentativas.
- Best-of-N: worktrees `.dare/worktrees/<id>` (branch `dare/cand-<id>`), seleção Pareto.
- Bench: fixtures (`suite.json`, `patch.diff`, `fail_to_pass.txt`, `pass_to_pass.txt`, `repo/`), Fix·Rate com zero em regressão de pass-to-pass.

#### 5.6 Hooks, Steering, Patterns/DNA
- **Hooks:** eventos fechados (`on-save`, `on-file-create`, `on-task-complete`, `pre-commit`); allowlist fechada de ações (`dare-validate`, `dare-review`, `graph-register`, `lint`, `test`); trust gate RS-05 (`trusted:false` default + `--trust`); idempotência por sha256.
- **Steering:** `DARE/PROJECT-DNA.md` e `DARE/PATTERNS.md` como base + `.dare/steering/*.md` com frontmatter `scope: project|glob`, `glob`, `priority`; `.env*` nunca elegível (RS-04).
- **Patterns:** mineração por frequência/coocorrência (kinds `inferred-layer, naming-idiom, structural-idiom, call-idiom, implicit-decision`) → `DARE/PATTERNS.md` + grafo.
- **DNA:** fatos de tooling/naming/arquitetura/testes/libraries/commits (git log) → `DARE/PROJECT-DNA.md`.

### 6. Servidores

#### 6.1 `dare-mcp-server`
**Atenção: não é servidor MCP de protocolo** — é Express REST puro (sem `@modelcontextprotocol/sdk`, sem JSON-RPC/stdio/SSE). `GET /tools` só *anuncia* 10 "tools" que são rotas REST:
`/health`, `/tools`, `POST /context/query`, `GET /blueprint`, `GET /dag`, `GET|PUT /tasks/:id` (PUT reescreve linha do TASKS.md por emoji), `POST /graph/locate|map-requirement|traverse`, `GET /project`, `GET /steering?file=`.
- Env: `DARE_MCP_BIND` (127.0.0.1), `DARE_MCP_PORT` (3000), `DARE_PROJECT_PATH`, `DARE_MCP_TOKEN` (UUID aleatório), `DARE_MCP_BODY_LIMIT` (1mb).
- Auth: Bearer token; **loopback passa sem token** por default; CORS hand-rolled (o pacote npm `cors` é dependência morta); helmet; path-safety em tudo (403 em escape).

#### 6.2 `dare dashboard`
Mesmo `createApp`; porta 4100; read-only: `GET /dashboard` (HTML vanilla de `templates/dashboard/`), `GET /api/telemetry` (`TelemetrySnapshot`: dag/gates/cost/bestOfN/guard/drift), assets com anti-traversal. Abre navegador automaticamente.

### 7. Integrações de IA

- **`src/ai/` (enrichment):** providers = CLIs de terminal **sem API key** (`codex` default, `claude-code` via `claude -p --output-format json`, `cursor-cli`, `antigravity-cli`, `mock`); overrides `DARE_{CODEX|CLAUDE|CURSOR|ANTIGRAVITY}_COMMAND`; timeout 20 min. Pipeline: heurística determinística sempre roda → LLM enriquece → validação Zod (8 schemas por comando) → injeção em marcadores `<!-- AGENT ... -->`.
- **`src/agent/` (drivers de execução):** interface `AgentDriver`; `claude` é o único uso do `@anthropic-ai/sdk` (`messages.create`, key em `ANTHROPIC_API_KEY` configurável via `agent.apiKeyEnv`, modelo default `claude-sonnet-4-5`, estimativa de custo por nome do modelo); `codex/cursor/antigravity` spawnam CLIs externos via safe-spawn; `mock/noop` para dry-run.

### 8. Arquivos canônicos (contratos de compatibilidade)

| Arquivo | Formato | Observação |
|---|---|---|
| `dare.config.json` | JSON, blocos parseados por subsistema (Zod strict, opt-in `enabled:false`) | preservar chaves desconhecidas (`flatten` no serde) |
| `dare-graph.yml` | YAML backend do grafo | |
| `DARE/dare-dag.yaml` | schema v2.1 + legado flat | |
| `.dare/state.json` | v1 | attempts com `failureSignature` |
| `.dare/graph.db` / `.dare/graph.json` | SQLite (schema §5.1) / JSON | BLOB vector f32 LE — compat binária |
| `.dare/skills.yml` | manifest de skills | header fixo |
| `.dare/verification/<id>.json` | baselines + proofs | |
| `DARE/.canvas.md`, `DARE/*.md` | markdown gerado | |
| `templates/UPDATE-MANIFEST.json` | schemaVersion 1 | |
| `~/.dare/registry/` | registry local | |

Invariantes críticos: IDs canônicos de nós/arestas (idempotência), `failureSignature` (normalização de stderr), ordenações determinísticas (localeCompare → definir ordenação byte-wise estável no port), path-safety (`"Error: path must be relative and stay within the project"`).

### 9. Dependências npm → crates Rust

| npm | Uso | Crate |
|---|---|---|
| commander | CLI parsing | **clap** (derive) |
| chalk | cores | **owo-colors** / anstyle |
| inquirer | prompts | **dialoguer** / inquire |
| ora | spinners | **indicatif** |
| figlet + gradient-string | banner | **figlet-rs** + colorgrad (interpolação manual) |
| fs-extra | I/O | std::fs + **fs_extra** |
| js-yaml + yaml | YAML (unificar) | **serde_yaml** |
| zod | validação | **serde** + garde/validator (+ schemars p/ JSON Schema) |
| handlebars / nunjucks / mustache | template engine por extensão (.hbs / .j2 .tera .erb / .mustache) | **handlebars** / **tera** (os `.tera` já existem!) / ramhorns |
| sql.js | grafo SQLite WASM | **rusqlite** (bundled; ganha FTS5 e elimina rewrite total) |
| @huggingface/transformers (opt) | embeddings MiniLM | **fastembed** (feature flag) ou ort+tokenizers |
| web-tree-sitter + tree-sitter-wasms (opt) | AST 8 gramáticas | **tree-sitter nativo** + crates de gramática |
| express + helmet + cors | servidores | **axum** + tower-http (cors é dep morta no original) |
| pino + pino-pretty | logging | **tracing** + tracing-subscriber |
| @anthropic-ai/sdk (opt) | 1 endpoint (messages.create) | **reqwest** + serde (superfície mínima) |
| node:crypto | sha256, Ed25519/minisign | **sha2**, **ed25519-dalek** |
| child_process | safe-spawn | **tokio::process** (kill-on-timeout) |

### 10. Riscos, bugs e decisões para o port

1. **Compat binária do grafo:** manter schema SQL + BLOB f32 LE, ou versionar e migrar (rusqlite permite abrir o `.dare/graph.db` existente diretamente).
2. **Idioma misto:** `update`, `review`, `refine` e partes do `execute` em português; resto em inglês. Padronizar?
3. **`--json` heterogêneo:** compacto em uns, pretty indent-2 em outros — definir contrato único.
4. **Bugs a decidir se replica ou corrige:** `dare welcome` menciona `dare new` (inexistente); mojibake `Â·` no discover; `skill update/remove` não tocam arquivos; publish remoto sem tarball; 5/6 skill.yml sem `dare_version`; UPDATE-MANIFEST sem releases 3.9+; codex ausente de `appliesTo` no manifest de update.
5. **"MCP server" que não é MCP:** decidir se o port mantém REST (compatível) ou implementa MCP real (rmcp) — os templates `mcp-*` gerados para projetos do usuário são servidores MCP de verdade; o do CLI não.
6. **Embeddings opcionais:** feature flag do Cargo (`--features semantic`) espelhando o optionalDependency.
7. **Windows:** npm-invoke evita `.cmd`; abrir navegador com `cmd /c start`; normalização de drive letter no code-index — cobrir com testes de plataforma.
8. **Testes de paridade de IDE** (`ide-command-parity.test.ts`) e checagem exaustiva dos 7 artefatos AX por stack: portar como testes de compilação/CI.


---

## Parte II — Arquitetura-alvo em Rust

## 11. Princípios de engenharia da reescrita

### 11.1 Fatias verticais

Cada ciclo precisa atravessar as camadas necessárias para entregar valor real:

```text
CLI → contratos → serviço de domínio → filesystem/execução → assets de IDE → testes → release
```

Um ciclo não será considerado concluído apenas porque um módulo compila. A entrega precisa ser instalável, exercitável em fixtures e publicada como binário beta.

### 11.2 Compatibilidade orientada por contratos

A unidade de compatibilidade não é a linha de código TypeScript. É o comportamento observável:

- comandos e aliases;
- argumentos e flags;
- mensagens essenciais;
- stdout e stderr;
- códigos de saída;
- arquivos criados, modificados e removidos;
- formatos JSON/YAML/SQLite;
- transições de estado;
- artefatos para os quatro harnesses;
- endpoints HTTP;
- invariantes de segurança.

### 11.3 Determinismo antes de IA

A filosofia atual do produto deve continuar:

1. executar análise e validação determinísticas;
2. chamar IA somente quando explicitamente habilitada ou quando o workflow exigir um agente;
3. validar qualquer retorno do modelo contra schemas;
4. não permitir que a IA altere contratos estruturais sem validação;
5. produzir os mesmos resultados determinísticos para a mesma fixture, salvo campos explicitamente voláteis.

### 11.4 Native-first

A nova versão não terá Node.js como requisito operacional. CLIs externos de agentes podem continuar sendo dependências opcionais do usuário, mas o `dare` deve funcionar como binário nativo autônomo.

### 11.5 Segurança por padrão

- Nunca construir comandos por concatenação de shell.
- Usar `argv` separado e environment allowlist.
- Canonicalizar e validar paths antes de leitura ou escrita.
- Tratar symlinks, junctions e drive letters do Windows.
- Fazer escrita atômica com backup quando houver sobrescrita.
- Remover secrets de logs e telemetria.
- Validar checksums e assinaturas de releases e skills remotas.
- Aplicar limites de corpo, timeout, fanout e profundidade.

---

## 12. Workspace Rust recomendado

```text
dare-rs/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── dare-cli/          # binário `dare`, clap e dispatch fino
│   ├── dare-core/         # erros, fs seguro, paths, processos, tracing
│   ├── dare-contracts/    # schemas persistidos e compatibilidade
│   ├── dare-config/       # carregamento, validação e migração de config
│   ├── dare-assets/       # assets canônicos, hashes e materialização
│   ├── dare-harness/      # Claude, Cursor, Codex e Antigravity
│   ├── dare-project/      # detecção brownfield, stacks e contexto
│   ├── dare-scaffold/     # init, bootstrap e geradores de stack
│   ├── dare-update/       # sync, backup, políticas e migrations
│   ├── dare-dag/          # DAG, ranks, estado, canvas e sub-DAG
│   ├── dare-graph/        # SQLite, JSON, Neo4j, RRF e traverse
│   ├── dare-ast/          # tree-sitter nativo e fallback regex
│   ├── dare-guard/        # unicode, injection scan e proveniência
│   ├── dare-verify/       # Ralph, mutation, formal, best-of-N e bench
│   ├── dare-agent/        # drivers e execução autônoma
│   ├── dare-ai/           # enrichment por CLI, schemas e marcadores
│   ├── dare-skills/       # registry, manifest, dependências e publish
│   ├── dare-hooks/        # hooks, trust gate e idempotência
│   ├── dare-steering/     # steering files e resolução por escopo
│   ├── dare-server/       # REST compatível, dashboard e MCP real
│   └── dare-telemetry/    # eventos, snapshots e custos
├── assets/
│   ├── capabilities/      # origem canônica dos workflows de IDE
│   ├── stacks/            # templates e metadados de scaffold
│   ├── dashboard/
│   ├── schemas/
│   ├── rules/
│   └── registry/
├── tests/
│   ├── fixtures/
│   ├── golden/
│   ├── compatibility/
│   ├── cross-platform/
│   └── security/
├── installers/
│   ├── install.sh
│   └── install.ps1
├── packaging/
│   ├── homebrew/
│   ├── winget/
│   └── scoop/
└── .github/workflows/
```

### 12.1 Regra de dependência

Crates de domínio não devem depender de `dare-cli`. O fluxo permitido será:

```text
dare-cli
  └── serviços de domínio
       └── contracts/core/config/assets
```

`dare-core` não deve virar um depósito geral. Schemas, assets, configuração, harnesses e update terão crates próprias.

### 12.2 Crates e bibliotecas sugeridas

| Necessidade | Crate sugerida |
|---|---|
| CLI e help | `clap`, `clap_complete`, `anstyle` |
| erros | `thiserror`, `anyhow` apenas nas bordas |
| async/processos | `tokio`, `tokio-util` |
| serialização | `serde`, `serde_json`, `serde_yaml` |
| schemas | `schemars`, `garde` ou `validator` |
| prompts | `dialoguer` ou `inquire` |
| progresso | `indicatif` |
| logging | `tracing`, `tracing-subscriber` |
| paths/globs | `camino`, `ignore`, `globset`, `walkdir` |
| Git | processo `git` com argv seguro; `git2` somente onde agregar valor |
| SQLite | `rusqlite` com `bundled` |
| grafo em memória | `petgraph` quando útil |
| HTTP client | `reqwest` + `rustls` |
| servidor | `axum`, `tower`, `tower-http` |
| AST | `tree-sitter` e gramáticas nativas |
| hashing | `sha2`, `blake3` quando não houver contrato SHA-256 |
| assinatura | `minisign-verify`, `ed25519-dalek` conforme formato escolhido |
| templates | `tera` e/ou `handlebars` conforme compatibilidade real |
| arquivos incorporados | `include_dir` ou `rust-embed` |
| semver/update | `semver`, `self_update` ou updater próprio auditável |
| testes CLI | `assert_cmd`, `predicates`, `insta` |
| diretórios temporários | `tempfile` |

---

## 13. Contratos persistidos

A crate `dare-contracts` deverá ser criada antes da maioria dos comandos. Ela será responsável por preservar e versionar os arquivos canônicos.

### 13.1 Tipos mínimos

```rust
pub struct DareConfig { /* campos conhecidos + flatten */ }
pub struct DagV21 { /* schema atual */ }
pub struct LegacyDag { /* schema flat */ }
pub struct RuntimeStateV1 { /* estados e attempts */ }
pub struct GraphNode { /* IDs canônicos */ }
pub struct GraphEdge { /* IDs canônicos */ }
pub struct SkillsManifest { /* .dare/skills.yml */ }
pub struct VerificationBaseline { /* proofs e aspectos */ }
pub struct UpdateManifestV1 { /* legado */ }
pub struct TelemetrySnapshot { /* dashboard */ }
```

### 13.2 Regras

- Preservar campos desconhecidos do `dare.config.json` com `#[serde(flatten)]`.
- Ler o DAG v2.1 e o schema legado.
- Não alterar `.dare/state.json` silenciosamente sem versionamento.
- Manter BLOB de vetores como `f32` little-endian enquanto a compatibilidade binária for exigida.
- Garantir ordenação determinística e independente de locale.
- Normalizar paths internos para `/`, preservando conversão correta no Windows.
- Definir writers canônicos para JSON e YAML; não depender de formatação acidental.
- Implementar migrations explícitas e reversíveis quando possível.

### 13.3 Política de compatibilidade

| Tipo de mudança | Política |
|---|---|
| Leitura de arquivo legado | obrigatória enquanto suportado |
| Escrita no formato legado | manter até ADR autorizar nova versão |
| Novo campo opcional | permitido com default seguro |
| Remoção/renomeação | somente com migration e changelog |
| Alteração de ID canônico | proibida sem migração integral |
| Alteração de exit code | tratada como breaking change |

---

## 14. Modelo de capabilities e suporte às IDEs

### 14.1 Duas camadas que não devem ser confundidas

**Skills-pacote:** gerenciadas por `dare skill`, com `skill.yml`, versão, dependências e registry.

**Capabilities de IDE:** arquivos de instrução, commands, rules e Agent Skills instalados em Claude, Cursor, Codex e Antigravity.

Uma skill de registry pode fornecer capabilities, mas os dois conceitos têm ciclos de vida e contratos diferentes.

### 14.2 Modelo canônico

```rust
pub struct Capability {
    pub id: CapabilityId,
    pub title: String,
    pub description: String,
    pub instructions: String,
    pub cli_commands: Vec<String>,
    pub outputs: HarnessOutputs,
    pub assets: Vec<AssetRef>,
}

pub struct HarnessOutputs {
    pub claude_command: Option<ClaudeCommand>,
    pub cursor_command: Option<CursorCommand>,
    pub cursor_rule: Option<CursorRule>,
    pub agent_skill: Option<AgentSkill>,
    pub project_instruction: Option<ProjectInstruction>,
}
```

Nem todo comando CLI precisa gerar uma skill. Comandos operacionais como `info` e `welcome` podem ser exclusivamente CLI. Workflows como `discover`, `design`, `blueprint`, `review` e `reverse` devem ter capabilities equivalentes nos harnesses relevantes.

### 14.3 Adapters

```rust
pub trait HarnessAdapter {
    fn id(&self) -> HarnessId;
    fn detect(&self, project: &ProjectRoot) -> Result<Detection>;
    fn plan_install(&self, capabilities: &[Capability]) -> Result<InstallPlan>;
    fn apply_install(&self, plan: &InstallPlan) -> Result<InstallReport>;
    fn validate(&self, project: &ProjectRoot) -> Result<ValidationReport>;
}
```

Implementações:

```text
ClaudeHarness
CursorHarness
CodexHarness
AntigravityHarness
```

### 14.4 Matriz de paridade obrigatória

Deverá existir um arquivo versionado, por exemplo `assets/capability-matrix.yml`:

```yaml
capabilities:
  dare-discover:
    cli: discover
    claude_command: true
    cursor_command: true
    cursor_rule: false
    codex_skill: true
    antigravity_skill: true
```

A CI deverá falhar quando:

- uma capability declarada não gerar o arquivo esperado;
- houver arquivo de IDE sem capability canônica ou exceção documentada;
- frontmatter for inválido;
- nomes forem duplicados;
- uma mudança reduzir paridade sem ADR.

### 14.5 Origem única dos assets

A nova implementação não deverá manter múltiplas árvores editáveis como fontes paralelas. A origem recomendada é:

```text
assets/capabilities/<capability-id>/
├── capability.yml
├── instructions.md
├── references/
└── templates/
```

O build gera:

```text
generated/claude/
generated/cursor/
generated/codex/
generated/antigravity/
```

Os outputs gerados são validados e incorporados ao binário.

---

## 15. Drivers de agentes e enrichment de IA

### 15.1 Separação obrigatória

```text
dare-harness = arquivos e integração do projeto com IDEs
dare-agent   = execução autônoma de agentes
dare-ai      = enrichment de documentos e análise assistida
```

### 15.2 Contrato de driver

```rust
#[async_trait]
pub trait AgentDriver {
    async fn doctor(&self) -> Result<DriverHealth>;
    async fn run(&self, request: AgentRequest, cancel: CancellationToken)
        -> Result<AgentRunResult>;
}
```

`AgentRequest` deverá carregar prompt, cwd/worktree, limites, environment allowlist, modelo, sandbox e política de aprovação. `AgentRunResult` deverá registrar status, resumo, stdout/stderr redigidos, tokens/custo quando disponíveis e evidências de execução.

### 15.3 Ordem de implementação dos drivers

1. `mock/noop` para testes.
2. Codex CLI.
3. Claude Code CLI.
4. Cursor Agent CLI.
5. Antigravity CLI.
6. Claude API direta, somente se continuar sendo requisito de produto.

A API direta não deverá ser misturada com o adapter Claude Code. Ela deve ser um driver separado e opcional.

---

## 16. Estratégia de distribuição nativa

### 16.1 Origem canônica

GitHub Releases será a origem oficial dos binários. Cada ciclo aceito gera uma versão beta ou alpha instalável.

```text
v0.1.0-alpha.1
v0.1.0-beta.1
v0.5.0-rc.1
v1.0.0
```

### 16.2 Targets mínimos

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-pc-windows-msvc
```

Avaliar adicionalmente `x86_64-unknown-linux-musl` para ambientes mínimos e containers.

### 16.3 Assets de release

```text
dare-v0.1.0-beta.1-x86_64-unknown-linux-gnu.tar.gz
dare-v0.1.0-beta.1-aarch64-unknown-linux-gnu.tar.gz
dare-v0.1.0-beta.1-x86_64-apple-darwin.tar.gz
dare-v0.1.0-beta.1-aarch64-apple-darwin.tar.gz
dare-v0.1.0-beta.1-x86_64-pc-windows-msvc.zip
SHA256SUMS
SHA256SUMS.sig
install.sh
install.ps1
SBOM.spdx.json
```

### 16.4 Canais

| Ambiente | Canal principal |
|---|---|
| Linux | `install.sh` ou asset fixado |
| macOS | Homebrew e `install.sh` |
| Windows | PowerShell + WinGet/Scoop |
| Desenvolvedor Rust | `cargo install` |
| CI/CD | asset versionado + checksum |
| Usuário legado | npm temporário ou documentação de transição |

### 16.5 Instalação

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://dare.dev/install.sh | sh
```

```powershell
irm https://dare.dev/install.ps1 | iex
```

O instalador deverá detectar plataforma, baixar o asset correto, verificar checksum/assinatura, instalar atomicamente e executar `dare --version`.

### 16.6 Auto-update

Implementar posteriormente:

```bash
dare self update --channel stable
dare self update --channel beta
dare self update --version 1.4.0
dare self rollback
dare self uninstall
```

Atualização deverá usar lock, download temporário, verificação, troca atômica e rollback.

### 16.7 Papel do npm

O npm não será parte da arquitetura final. Durante a transição, `@dewtech/dare-cli` pode:

- continuar entregando a versão TypeScript sob tag `legacy`;
- emitir aviso de migração;
- apontar para os instaladores nativos;
- ser descontinuado após o cutover.

A versão Rust não dependerá de npm nem de Node.js.

---

## Parte III — Roadmap incremental

## 17. Convenções dos ciclos

Cada ciclo terá:

- objetivo de produto;
- comandos e subcomandos;
- crates envolvidas;
- capabilities para IDE quando aplicável;
- contratos de disco afetados;
- testes de paridade;
- release nativo;
- critérios de aceite.

Versões abaixo são indicativas. O importante é manter releases cumulativos e instaláveis.

---

## 18. Ciclo 0 — Fundação, contratos e baseline de paridade

**Objetivo:** preparar o workspace e impedir que a reescrita avance sem contratos mensuráveis.

### Entregas

- workspace Rust e políticas de lint/format;
- `dare-core`, `dare-contracts`, `dare-config`, `dare-assets`;
- harness de teste TypeScript × Rust;
- inventário de assets e capability matrix;
- fixtures canônicas;
- pipeline de build dos cinco targets mínimos;
- geração de checksums e SBOM;
- snapshot do comportamento do CLI 3.18.1.

### Fixtures mínimas

```text
empty-project
existing-node-project
existing-rust-project
existing-python-project
monorepo
project-with-claude
project-with-cursor
project-with-codex
project-with-antigravity
project-with-all-harnesses
invalid-config
legacy-dag
customized-assets
windows-path-cases
```

### Critério de aceite

- binário Rust responde `--version` e `--help`;
- CI cross-platform produz assets de teste;
- schemas principais leem fixtures legadas;
- golden runner consegue invocar as duas implementações e normalizar resultados.

---

## 19. Ciclo 1 — Primeiro CLI útil: `welcome`, `info` e `discover`

**Release sugerido:** `v0.1.0-alpha.1`

### Comandos

```bash
dare welcome
dare info
dare discover
dare discover --check
```

### Por que `discover` primeiro

`discover` instala o DARE em um projeto existente. `init` é scaffolding greenfield e depende de stacks, templates e toolchains; por isso será implementado em fase posterior.

### Entregas

- descoberta do root do projeto e Git;
- detecção básica de stack;
- geração de `dare.config.json` compatível;
- criação de `DARE/`, `.dare/`, templates e graph config;
- instalação idempotente dos harnesses;
- suporte a Claude, Cursor, Codex e Antigravity;
- `--check` sem mutação;
- mensagens e relatórios claros;
- correção do bug textual `dare new` no `welcome`, documentada como incompatibilidade intencional.

### Capabilities

- `dare-discover` para os harnesses aplicáveis;
- instruções de projeto raiz;
- base comum DARE workflow.

### Aceite

- executar duas vezes não duplica nem corrompe arquivos;
- `--check` retorna resultado sem escrever;
- cada harness gera exatamente os arquivos declarados na capability matrix;
- projeto com todos os harnesses continua válido.

---

## 20. Ciclo 2 — `validate`

**Release sugerido:** `v0.2.0-alpha.1`

### Comando

```bash
dare validate
dare validate --strict
dare validate --dag <path>
```

### Entregas

- parser DAG v2.1 e legado;
- IDs únicos e kebab-case;
- referências de dependência;
- detecção de ciclos;
- prompts e specs obrigatórios;
- limites e warnings;
- códigos de saída compatíveis;
- saída humana e JSON estável.

### Aceite

- fixtures válidas e inválidas reproduzem o resultado esperado;
- ordenação dos erros é determinística;
- nenhum erro de validação causa escrita em disco.

---

## 21. Ciclo 3 — `update` e gestão de assets

**Release sugerido:** `v0.3.0-alpha.1`

### Comando

```bash
dare update
dare update --dry-run
dare update --force
dare update --target <harness>
```

### Entregas

- manifesto de assets gerenciados;
- SHA-256 e classificação `identical|missing|apply|customized`;
- políticas `keep|replace|ask`;
- backup em `.dare/backup-*`;
- atualização seletiva de harness;
- Codex incluído explicitamente;
- migrations de config;
- relatório JSON.

### Decisão

Criar um novo manifest versionado, mas manter leitor de `UPDATE-MANIFEST.json` schema 1. O bug de ausência das versões 3.9+ não será reproduzido.

---

## 22. Ciclo 4 — `design`

**Release sugerido:** `v0.4.0-alpha.1`

### Comandos

```bash
dare design "descrição"
dare design --interactive
dare design --ai --provider <provider>
```

### Entregas

- geração determinística de `DARE/DESIGN.md`;
- markers de enrichment;
- providers `mock` e pelo menos um provider CLI real;
- validação de schema;
- capability `dare-design` nos quatro harnesses;
- preservação de conteúdo personalizado fora dos markers gerenciados.

---

## 23. Ciclo 5 — `blueprint`

**Release sugerido:** `v0.5.0-alpha.1`

### Comandos

```bash
dare blueprint
dare blueprint <design>
dare blueprint --force
```

### Entregas

- `BLUEPRINT.md`;
- `TASKS.md`;
- `DARE/dare-dag.yaml`;
- diretório `DARE/EXECUTION/`;
- templates canônicos;
- capability `dare-blueprint`;
- validação do DAG recém-gerado.

---

## 24. Ciclo 6 — DAG e visualização

**Release sugerido:** `v0.6.0-alpha.1`

### Comando

```bash
dare dag viz --format mermaid
dare dag viz --format dot
dare dag viz --format excalidraw
```

### Entregas

- cálculo de ranks;
- detecção de ciclos reutilizada;
- estado runtime v1;
- canvas;
- visualizações determinísticas;
- writers estáveis para Mermaid, DOT e Excalidraw.

---

## 25. Ciclo 7 — `execute` determinístico

**Release sugerido:** `v0.7.0-alpha.1`

### Ações

```bash
dare execute --status
dare execute --next
dare execute --watch
dare execute --complete <id>
dare execute --fail <id>
dare execute --reset <id>
```

### Escopo

Sem agente autônomo neste ciclo.

### Entregas

- state store;
- cascading skip;
- seleção do menor rank executável;
- composição de prompt;
- contexto dos pais com cap;
- canvas;
- transições atômicas;
- ingestão básica pós-DONE;
- Ralph Loop inicial com build/test/lint.

### Aceite

- crash não deixa state parcialmente escrito;
- duas execuções concorrentes usam lock ou falham claramente;
- códigos de timeout e falha são preservados.

---

## 26. Ciclo 8 — `execute --agent` com mock e infraestrutura

**Release sugerido:** `v0.8.0-alpha.1`

### Entregas

- contrato `AgentDriver`;
- driver `mock/noop`;
- worktrees;
- budget tracker;
- timeout/cancelamento;
- tentativas e failure signatures;
- guard preflight;
- logs e telemetria;
- política fixa inicial.

Este ciclo valida a máquina de estados sem introduzir variabilidade de agentes reais.

---

## 27. Ciclo 9 — Drivers reais Claude, Codex, Cursor e Antigravity

**Release sugerido:** `v0.9.0-beta.1`

### Entregas

- Codex CLI JSONL;
- Claude Code CLI;
- Cursor Agent CLI;
- Antigravity CLI;
- `doctor` por driver;
- environment overrides compatíveis;
- sandbox e approval policy quando suportados;
- normalização de resultados;
- redaction de secrets.

### Aceite

Cada driver passa por uma suite comum:

```text
detection
version/doctor
success
failure
timeout
cancellation
malformed output
missing executable
secret redaction
```

---

## 28. Ciclo 10 — `review`

### Comando

```bash
dare review <task-id>
```

### Entregas

- anti-stub, mock e TODO;
- severidades;
- formatos human, JSON e GitHub;
- `--strict`, `--errors-only`, `--files`, `--from-agent`, `--comment`, `--fail-on`;
- enrichment opcional;
- capability `dare-review`.

---

## 29. Ciclo 11 — `refine` e sub-DAG

### Comando

```bash
dare refine <task-id>
```

### Entregas

- medição de complexidade;
- proposta de split;
- `--apply`;
- `spliceSubDag`;
- limites de profundidade;
- cycle protection;
- exit code 2 no modo estrito para HIGH/CRITICAL;
- capability `dare-refine`.

---

## 30. Ciclo 12 — `guard`

### Comando

```bash
dare guard [target]
dare guard --staged
dare guard --all
dare guard --sign
```

### Entregas

- Unicode scan;
- prompt-injection rules;
- proveniência;
- trusted paths;
- assinatura minisign/Ed25519;
- redaction de evidências;
- exit code 6;
- integração como preflight do agente.

---

## 31. Ciclo 13 — Brownfield: `reverse`

### Comando

```bash
dare reverse
```

### Entregas

- análise de módulos;
- relatório e modo profundo;
- geração de IDEIA/specs;
- AST nativo inicial;
- fallback regex;
- Excalidraw opcional;
- enrichment;
- capability `dare-reverse`.

---

## 32. Ciclo 14 — Brownfield: `dna`, `patterns` e `migrate`

Implementar em subciclos independentes se necessário.

### Comandos

```bash
dare dna
dare patterns
dare migrate --to <stack>
```

### Entregas

- PROJECT-DNA;
- PATTERNS;
- fatos de Git/tooling/naming;
- mineração determinística;
- injeção de padrões;
- plano de migração;
- Gherkin de paridade;
- capabilities correspondentes.

---

## 33. Ciclo 15 — GraphRAG básico

### Subcomandos

```bash
dare graph ingest
dare graph query
dare graph stats
dare graph viz
```

### Entregas

- backend SQLite compatível;
- backend JSON;
- schema e IDs canônicos;
- ingestão incremental por content hash;
- keyword search;
- BFS;
- RRF;
- visualização;
- feature `semantic` opcional.

### Compatibilidade crítica

Abrir uma cópia de `.dare/graph.db` legado, executar leitura e mutação em Rust e confirmar schema, BLOB e IDs.

---

## 34. Ciclo 16 — GraphRAG avançado

### Subcomandos

```bash
dare graph owners
dare graph impact
dare graph trace
dare graph locate
dare graph drift
```

### Entregas

- locate com seeds e decay;
- impact e owners;
- trace;
- drift;
- threshold e exit code 7;
- Neo4j experimental em subciclo posterior se necessário.

---

## 35. Ciclo 17 — Skills registry

### Subcomandos

```bash
dare skill list
dare skill info
dare skill add
dare skill remove
dare skill update
dare skill publish
```

### Entregas

- registry mock, local e remoto;
- resolução topológica;
- manifest e lockfile novo opcional;
- instalação atômica;
- remoção real de arquivos;
- update real de conteúdo;
- publish com artefato, hash e assinatura;
- reader compatível com manifest legado.

### Incompatibilidades intencionais

Os bugs de `remove`, `update` e publish apenas de metadados serão corrigidos e documentados.

---

## 36. Ciclo 18 — `init` e `bootstrap`

### Comandos

```bash
dare init [nome]
dare bootstrap
```

### Por que entram aqui

Dependem de contratos, assets, stacks, templates, prompts, update e harnesses já consolidados.

### Entregas

- modo interativo e `--non-interactive`;
- stacks backend e frontend;
- MCP stacks;
- toolchains;
- transport;
- fullstack;
- sete artefatos AX por stack;
- rollback em scaffold parcial;
- bootstrap idempotente.

### Stacks mínimas de paridade

```text
ruby-rails-8
node-nestjs
python-fastapi
php-laravel
rust-axum
go-gin
go-stdlib
mcp-node-ts
mcp-python
mcp-rust
mcp-go
```

---

## 37. Ciclo 19 — Hooks e steering

### Comandos

```bash
dare hooks list
dare hooks run <evento>
dare hooks validate
dare steering list
dare steering show <file>
```

### Entregas

- eventos fechados;
- allowlist de ações;
- trust gate;
- idempotência SHA-256;
- steering por frontmatter, glob e prioridade;
- exclusão obrigatória de `.env*`.

---

## 38. Ciclo 20 — Verificação avançada e `bench`

### Comandos e recursos

```bash
dare bench
dare execute --best-of <n>
dare execute --full-mutation
dare execute --formal
dare execute --policy decay
```

### Entregas

- fail-to-pass;
- anti-tamper;
- mutation adapters;
- Dafny, Verus e Lean;
- repair loop;
- best-of-N;
- seleção Pareto;
- decay/replan/escalate;
- bench e regressão de baseline.

---

## 39. Ciclo 21 — `ai`

### Subcomandos

```bash
dare ai doctor
dare ai providers
dare ai run
dare ai prompt
```

### Entregas

- providers terminal-first;
- schemas dos oito workflows;
- markers;
- timeouts;
- mock;
- diagnostics;
- configuração explícita de provider.

---

## 40. Ciclo 22 — Dashboard, REST compatível e MCP real

### Entregas

1. Dashboard read-only em Axum.
2. Endpoints REST compatíveis com o servidor legado.
3. Transporte MCP real separado.

### Comandos sugeridos

```bash
dare dashboard
dare server --protocol rest
dare mcp serve --transport stdio
dare mcp serve --transport streamable-http
```

Para compatibilidade, o binário `dare-mcp-server` pode permanecer como alias ou wrapper durante uma janela de transição.

### Segurança

- bind loopback por padrão;
- token obrigatório fora de loopback;
- body limit;
- CORS explícito;
- headers de segurança;
- path safety;
- shutdown gracioso.

---

## 41. Ciclo 23 — Self-update, empacotamento e candidato 1.0

### Entregas

- `dare self update`;
- Homebrew tap;
- WinGet ou Scoop;
- instaladores estáveis;
- assinatura de releases;
- SBOM;
- rollback;
- documentação de migração;
- telemetria opt-in ou inexistente por padrão;
- release candidate.

---

## Parte IV — Qualidade, segurança e cutover

## 42. Testes de paridade TypeScript × Rust

### 42.1 Dimensões comparadas

```text
exit code
stdout
stderr
file tree
file contents
JSON/YAML semântico
SQLite schema e rows
state transitions
IDE artifacts
HTTP responses
hashes e assinaturas
```

### 42.2 Normalização permitida

Somente campos declarados como voláteis:

```text
timestamps
UUIDs/tokens
paths temporários
cores ANSI
separadores de path
drive-letter casing
versão do binário
```

Não normalizar diferenças que escondam quebra de contrato.

### 42.3 Tipos de testes

- unitários por crate;
- property tests para DAG, paths e serialization;
- golden/snapshot;
- integração em filesystem real;
- cross-platform;
- security regression;
- fuzzing de parsers e path inputs;
- compatibility database tests;
- smoke tests dos instaladores;
- end-to-end com agentes mock;
- contract tests dos drivers reais.

### 42.4 Testes de segurança prioritários

- path traversal e symlink escape;
- command injection;
- environment leak;
- malformed YAML/JSON;
- zip/tar traversal em registries/releases;
- signature mismatch;
- oversized HTTP body;
- cancellation e orphan process;
- concurrent state writes;
- malicious agent output;
- Unicode bidi e homoglyphs.

---

## 43. Definition of Done por ciclo

```text
[ ] comando/subcomando implementado
[ ] help, aliases e flags documentados
[ ] saída humana revisada
[ ] saída JSON versionada ou estável
[ ] exit codes preservados/decididos
[ ] contratos de disco cobertos
[ ] capability Claude quando aplicável
[ ] capability Cursor quando aplicável
[ ] capability Codex quando aplicável
[ ] capability Antigravity quando aplicável
[ ] testes unitários
[ ] testes de integração
[ ] golden tests contra TypeScript
[ ] fixture greenfield quando aplicável
[ ] fixture brownfield quando aplicável
[ ] Linux validado
[ ] macOS validado
[ ] Windows validado
[ ] segurança e path-safety revisadas
[ ] release notes
[ ] binários e checksums publicados
[ ] instalação limpa validada
[ ] upgrade da versão anterior validado
```

---

## 44. Política para bugs e incompatibilidades

### Classe A — Contrato público

Preservar, salvo breaking change aprovada:

- exit codes;
- nomes e flags;
- schemas persistidos;
- IDs canônicos;
- comportamento usado por CI.

### Classe B — Bug cosmético

Corrigir diretamente, documentando:

- `dare new` inexistente no welcome;
- mojibake;
- formatação inconsistente.

### Classe C — Bug comportamental potencialmente utilizado

Exigir ADR e migration note:

- skill update/remove incompletos;
- diferenças de JSON;
- idioma misto;
- policies de update.

### Classe D — Vulnerabilidade

Corrigir obrigatoriamente, mesmo com mudança de comportamento:

- path escape;
- execução insegura;
- secret leakage;
- assinatura ausente ou inválida;
- extração de arquivo insegura.

---

## 45. ADRs obrigatórias antes do beta

```text
ADR-001 — Compatibilidade de bugs legados
ADR-002 — Contrato de saída JSON
ADR-003 — Idioma da CLI
ADR-004 — REST compatível e MCP real
ADR-005 — Protocolo e formato do skill registry
ADR-006 — Compatibilidade e migração do Graph DB
ADR-007 — Formato canônico de capabilities
ADR-008 — Assinatura de releases e skills
ADR-009 — Política de auto-update
ADR-010 — Claude API direta versus CLI
ADR-011 — Telemetria, privacidade e opt-in
ADR-012 — Estratégia de versionamento dos contratos de disco
```

---

## 46. Estratégia de branches e releases

### Branches

```text
main                 # versão estável ou candidata
rust/main            # integração da reescrita, se mesmo repositórioeature/<capability> # fatias verticais curtas
```

Alternativamente, manter um repositório Rust separado durante a fase alpha e incorporar somente quando os contratos e pipelines estiverem maduros.

### Canais

```text
alpha  = arquitetura em evolução, contratos ainda podem mudar
beta   = comandos utilizáveis, migração assistida
rc     = paridade completa, correções apenas
stable = substituição oficial
```

### Regra de release

Todo ciclo aceito produz:

- tag;
- binários;
- checksums;
- assinatura;
- SBOM;
- changelog;
- matriz de compatibilidade;
- instruções de instalação e rollback.

---

## 47. Cutover da versão TypeScript

### 47.1 Pré-requisitos

- matriz de comandos completa;
- contratos de disco testados em cópias de projetos reais;
- todos os harnesses aprovados;
- drivers reais em estado suportado;
- releases nativos estáveis;
- updater e rollback validados;
- documentação de migração;
- incident response e política de security advisories.

### 47.2 Estratégia

1. Publicar beta nativa em paralelo.
2. Executar shadow tests e projetos piloto.
3. Marcar feature freeze da TypeScript, exceto segurança.
4. Publicar RC e bloquear mudanças de contrato.
5. Tornar Rust a versão recomendada.
6. Mover npm TypeScript para `legacy`.
7. Manter janela de correções críticas.
8. Arquivar ou remover o canal legado conforme política publicada.

### 47.3 Rollback

- instaladores preservam versão anterior;
- `dare self rollback` restaura o binário;
- migrations destrutivas exigem backup;
- formatos novos devem continuar legíveis pela mesma major;
- release pode ser revogada sem quebrar URLs de versões anteriores.

---

## 48. Métricas de acompanhamento

### Produto

- comandos completos / comandos totais;
- capabilities completas por harness;
- stacks completas;
- projetos piloto ativos;
- taxa de sucesso de upgrade.

### Compatibilidade

- golden cases aprovados;
- diferenças intencionais abertas;
- contratos de disco aprovados;
- paridade de IDE;
- endpoints REST/MCP cobertos.

### Engenharia

- tempo de startup;
- tamanho dos binários;
- duração de scans;
- consumo de memória;
- cobertura por crate;
- vulnerabilidades abertas;
- flakiness cross-platform.

### Qualidade de execução

- tasks concluídas;
- falhas por gate;
- timeouts;
- fresh-start/replan/escalate;
- custo/token por driver quando disponível;
- regressões de bench.

---

## 49. Próximas ações concretas

1. Aprovar este documento como baseline e roadmap.
2. Criar os ADRs 001, 002, 004, 006 e 007 antes de implementar domínio complexo.
3. Gerar a capability matrix completa a partir dos 49 comandos Claude, 33 commands Cursor, 25 rules Cursor e 48 Agent Skills.
4. Criar fixtures e capturar golden outputs da v3.18.1.
5. Criar o workspace Rust e as crates `core`, `contracts`, `config` e `assets`.
6. Montar pipeline cross-platform de releases alpha.
7. Implementar o Ciclo 1 com `welcome`, `info` e `discover`.
8. Testar instalação simultânea em Claude, Cursor, Codex e Antigravity.
9. Publicar o primeiro release nativo alpha.
10. Só então avançar para `validate` e `update`.

---

## 50. Conclusão

O mapeamento da versão 3.18.1 define a realidade técnica atual. O roadmap incremental define como substituí-la sem perder comportamento, ativos de IDE ou segurança operacional.

A estratégia recomendada não é uma tradução arquivo por arquivo. É uma reconstrução baseada em contratos, capabilities e fatias verticais. O primeiro valor é entregue cedo com `discover`; os componentes mais complexos, especialmente `execute`, GraphRAG, skills registry e scaffolding, entram somente quando suas fundações estiverem testadas.

O resultado final será um DARE CLI nativo, distribuído sem npm, com inicialização rápida, filesystem e SQLite nativos, AST por tree-sitter, execução segura de processos, releases assinados e suporte consistente a Claude Code, Cursor, Codex e Antigravity.
