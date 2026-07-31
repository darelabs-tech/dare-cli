# DARE CLI Rust — Sequencia de Microplanejamento

> Derivado do Documento Mestre da reescrita do DARE CLI em Rust.
> Execute os arquivos em ordem numerica. Um microplano so deve iniciar quando os criterios de aceite dos seus pre-requisitos estiverem satisfeitos.

## Como usar

1. Abra o arquivo de menor numero ainda nao concluido.
2. Transforme as tarefas em issues ou tickets tecnicos.
3. Marque a checklist no proprio arquivo ou no tracker do projeto.
4. Nao avance quando um criterio de aceite estiver pendente.
5. Registre diferencas de compatibilidade em ADR antes de altera-las.

## Sequencia

- [001 — Governanca, baseline e ADRs prioritarias](001-governanca-baseline-e-adrs-prioritarias.md)
- [002 — Workspace Rust e toolchain](002-workspace-rust-e-toolchain.md)
- [003 — CI cross-platform e qualidade](003-ci-cross-platform-e-qualidade.md)
- [004 — Erros, tracing e saida da CLI](004-erros-tracing-e-saida-da-cli.md)
- [005 — Filesystem seguro e path safety](005-filesystem-seguro-e-path-safety.md)
- [006 — Execucao segura de processos](006-execucao-segura-de-processos.md)
- [007 — Contratos persistidos](007-contratos-persistidos.md)
- [008 — Configuracao e migrations](008-configuracao-e-migrations.md)
- [009 — Inventario e empacotamento de assets](009-inventario-e-empacotamento-de-assets.md)
- [010 — Modelo canonico de capabilities](010-modelo-canonico-de-capabilities.md)
- [011 — Adapter Claude Code](011-adapter-claude-code.md)
- [012 — Adapter Cursor](012-adapter-cursor.md)
- [013 — Adapter Codex](013-adapter-codex.md)
- [014 — Adapter Antigravity](014-adapter-antigravity.md)
- [015 — Pipeline de release nativo alpha](015-pipeline-de-release-nativo-alpha.md)
- [016 — Comando welcome](016-comando-welcome.md)
- [017 — Comando info](017-comando-info.md)
- [018 — Discover: deteccao brownfield](018-discover-deteccao-brownfield.md)
- [019 — Discover: instalacao do DARE](019-discover-instalacao-do-dare.md)
- [020 — Validate](020-validate.md)
- [021 — Update: planejamento e manifest](021-update-planejamento-e-manifest.md)
- [022 — Update: aplicacao, backup e migrations](022-update-aplicacao-backup-e-migrations.md)
- [023 — Design deterministico](023-design-deterministico.md)
- [024 — Fundacao de enrichment por IA](024-fundacao-de-enrichment-por-ia.md)
- [025 — Blueprint](025-blueprint.md)
- [026 — DAG: parser, ranks e state store](026-dag-parser-ranks-e-state-store.md)
- [027 — DAG: visualizacao](027-dag-visualizacao.md)
- [028 — Execute: status, next e watch](028-execute-status-next-e-watch.md)
- [029 — Execute: complete, fail, reset e Ralph inicial](029-execute-complete-fail-reset-e-ralph-inicial.md)
- [030 — Execute agent: mock, worktrees e budget](030-execute-agent-mock-worktrees-e-budget.md)
- [031 — Drivers reais de agentes](031-drivers-reais-de-agentes.md)
- [032 — Review](032-review.md)
- [033 — Refine e sub-DAG](033-refine-e-sub-dag.md)
- [034 — Guard](034-guard.md)
- [035 — Engine AST nativo](035-engine-ast-nativo.md)
- [036 — Reverse](036-reverse.md)
- [037 — DNA](037-dna.md)
- [038 — Patterns](038-patterns.md)
- [039 — Migrate](039-migrate.md)
- [040 — GraphRAG: storage e compatibilidade](040-graphrag-storage-e-compatibilidade.md)
- [041 — GraphRAG: ingest, keyword, BFS e RRF](041-graphrag-ingest-keyword-bfs-e-rrf.md)
- [042 — GraphRAG semantico opcional](042-graphrag-semantico-opcional.md)
- [043 — GraphRAG avancado e Neo4j](043-graphrag-avancado-e-neo4j.md)
- [044 — Skills registry: modelo e resolucao](044-skills-registry-modelo-e-resolucao.md)
- [045 — Skills lifecycle e publish seguro](045-skills-lifecycle-e-publish-seguro.md)
- [046 — Scaffolding: contratos, stacks e artefatos AX](046-scaffolding-contratos-stacks-e-artefatos-ax.md)
- [047 — Init e bootstrap](047-init-e-bootstrap.md)
- [048 — Hooks e steering](048-hooks-e-steering.md)
- [049 — Verificacao avancada e bench](049-verificacao-avancada-e-bench.md)
- [050 — Comandos ai](050-comandos-ai.md)
- [051 — Dashboard e REST compativel](051-dashboard-e-rest-compativel.md)
- [052 — MCP real como transporte separado](052-mcp-real-como-transporte-separado.md)
- [053 — Self-update e package managers](053-self-update-e-package-managers.md)
- [054 — Hardening de paridade e seguranca](054-hardening-de-paridade-e-seguranca.md)
- [055 — Pilotos, shadow tests e release candidate](055-pilotos-shadow-tests-e-release-candidate.md)
- [056 — Cutover, stable e encerramento do legado](056-cutover-stable-e-encerramento-do-legado.md)