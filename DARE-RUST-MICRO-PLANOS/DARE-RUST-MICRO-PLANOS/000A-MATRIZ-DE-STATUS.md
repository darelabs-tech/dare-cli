# Matriz resumida de execucao

| Seq. | Microplano | Status | Dependencia principal |
|---:|---|---|---|
| 001 | Governanca, baseline e ADRs prioritarias | ⬜ Pendente | Documento mestre aprovado |
| 002 | Workspace Rust e toolchain | ⬜ Pendente | Microplano 001 concluido |
| 003 | CI cross-platform e qualidade | ⬜ Pendente | Microplano 002 concluido |
| 004 | Erros, tracing e saida da CLI | ⬜ Pendente | Microplano 002 concluido |
| 005 | Filesystem seguro e path safety | ⬜ Pendente | Microplanos 002 e 004 concluidos |
| 006 | Execucao segura de processos | ⬜ Pendente | Microplanos 002 e 004 concluidos |
| 007 | Contratos persistidos | ⬜ Pendente | Microplanos 002, 004 e 005 concluidos |
| 008 | Configuracao e migrations | ⬜ Pendente | Microplano 007 concluido |
| 009 | Inventario e empacotamento de assets | ⬜ Pendente | Microplanos 001 e 002 concluidos |
| 010 | Modelo canonico de capabilities | ⬜ Pendente | Microplano 009 concluido |
| 011 | Adapter Claude Code | ⬜ Pendente | Microplanos 005, 009 e 010 concluidos |
| 012 | Adapter Cursor | ⬜ Pendente | Microplanos 005, 009 e 010 concluidos |
| 013 | Adapter Codex | ⬜ Pendente | Microplanos 005, 009 e 010 concluidos |
| 014 | Adapter Antigravity | ⬜ Pendente | Microplanos 005, 009 e 010 concluidos |
| 015 | Pipeline de release nativo alpha | ⬜ Pendente | Microplano 003 concluido |
| 016 | Comando welcome | ⬜ Pendente | Microplanos 004 e 015 concluidos |
| 017 | Comando info | ⬜ Pendente | Microplanos 007 a 015 concluidos |
| 018 | Discover: deteccao brownfield | ⬜ Pendente | Microplanos 005, 007, 008 e 009 concluidos |
| 019 | Discover: instalacao do DARE | ⬜ Pendente | Microplanos 011 a 014 e 018 concluidos |
| 020 | Validate | ⬜ Pendente | Microplanos 004, 007 e 008 concluidos |
| 021 | Update: planejamento e manifest | ⬜ Pendente | Microplanos 008 a 014 concluidos |
| 022 | Update: aplicacao, backup e migrations | ⬜ Pendente | Microplano 021 concluido |
| 023 | Design deterministico | ✅ Concluido | Microplanos 009, 010 e 019 concluidos |
| 024 | Fundacao de enrichment por IA | ✅ Concluido | Microplanos 006 e 023 concluidos |
| 025 | Blueprint | ✅ Concluido | Microplanos 020, 023 e 024 concluidos |
| 026 | DAG: parser, ranks e state store | ✅ Concluido | Microplanos 005, 007 e 020 concluidos |
| 027 | DAG: visualizacao | ✅ Concluído | `dare dag viz` Mermaid/DOT/Excalidraw + DEC-028 |
| 028 | Execute: status, next e watch | ✅ Concluído | `dare execute --status|--next|--watch` + DEC-029 |
| 029 | Execute: complete, fail, reset e Ralph inicial | ✅ Concluído | `dare-verify` + `--complete/--fail/--reset` + DEC-030 |
| 030 | Execute agent: mock, worktrees e budget | ✅ Concluído | Crate `dare-agent`; CLI `--agent`/`--cleanup-worktrees`; DEC-031; docs `cli-execute-agent.md` |
| 031 | Drivers reais de agentes | ✅ Concluído | Drivers CLI + resolve; DEC-037 + docs `cli-execute-agent.md`; fechamento mp031-008 |
| 032 | Review | ✅ Concluído | Crate `dare-review`; CLI `dare review`; DEC-034; docs `cli-review.md` |
| 033 | Refine e sub-DAG | ✅ Concluído | `dare refine` + `dare-dag::subdag`; DEC-040; docs `cli-refine.md` |
| 034 | Guard | ✅ Concluído | Microplanos 005 e 006; DEC-035; exit 6 + preflight agent |
| 035 | Engine AST nativo | ✅ Concluído | Crate `dare-ast`; tree-sitter nativo + regex fallback; DEC-032; docs `ast-engine.md` |
| 036 | Reverse | ✅ Concluído | `dare reverse`; IDEIA/REVERSE; --check; DEC-038 |
| 037 | DNA | ✅ Concluído | `dare dna` + PROJECT-DNA; DEC-039 + docs `cli-dna.md` |
| 038 | Patterns | ✅ Concluído | `dare patterns` + PATTERNS.md; DEC-041 + docs `cli-patterns.md` |
| 039 | Migrate | ✅ Concluído | `dare migrate` + DEC-044 + docs `cli-migrate.md` |
| 040 | GraphRAG: storage e compatibilidade | ✅ Concluído | Crate `dare-graph`; SQLite+JSON; IDs+BLOB f32 LE; DEC-036 |
| 041 | GraphRAG: ingest, keyword, BFS e RRF | ✅ Concluído | ingest+search+RRF; CLI `dare graph *`; DEC-042; docs `graphrag-ingest.md` |
| 042 | GraphRAG semantico opcional | ✅ Concluído | feature `semantic`+fastembed; doctor/enable; DEC-045; docs `graphrag-semantic.md` |
| 043 | GraphRAG avancado e Neo4j | ✅ Concluído | locate/owners/impact/trace/drift exit 7; feature `neo4j` opt-in; DEC-046; docs `graphrag-advanced.md` |
| 044 | Skills registry: modelo e resolucao | ✅ Concluído | Microplanos 005, 007 e 009 concluidos |
| 045 | Skills lifecycle e publish seguro | ✅ Concluído | add/remove/update/publish; DEC-043; path jail; docs cli-skill |
| 046 | Scaffolding: contratos, stacks e artefatos AX | ✅ Concluído | crate `dare-scaffold`; DEC-047; docs `scaffold-contracts.md` |
| 047 | Init e bootstrap | ✅ Concluído | CLI `dare init`/`dare bootstrap`; DEC-048; docs `cli-init-bootstrap.md` |
| 048 | Hooks e steering | ✅ Concluído | Microplanos 005, 006 e 019 concluidos |
| 049 | Verificacao avancada e bench | ✅ Concluído | `dare-verify` advanced + `dare bench`; DEC-050; docs `cli-verify-bench.md` |
| 050 | Comandos ai | ✅ Concluído | `dare ai` doctor/providers/run/prompt; DEC-051; docs `cli-ai.md` |
| 051 | Dashboard e REST compativel | ✅ Concluído | `dare-server` Axum + CLI dashboard/server; DEC-052; docs `cli-dashboard-rest.md` |
| 052 | MCP real como transporte separado | ✅ Concluído | `dare mcp serve` stdio/streamable-http + rmcp; DEC-053; docs `cli-mcp.md` |
| 053 | Self-update e package managers | ✅ Concluído | crate `dare-self` + CLI `dare self`; DEC-054; docs `cli-self-update.md`; packaging Homebrew+WinGet |
| 054 | Hardening de paridade e seguranca | ✅ Concluído | crate `dare-parity` + golden/security; DEC-055; docs `parity-hardening.md`; gate 15% |
| 055 | Pilotos, shadow tests e release candidate | ✅ Concluído | docs/pilot + freeze TS/contrato; RC `v4.0.0-rc1`; DEC-056; rollback PASS; matrix 51 |
| 056 | Cutover, stable e encerramento do legado | ⬜ Pendente | Microplano 055 concluido |