# ROADMAP — DARE Harness Evolution and Observability

> 2026-10-22 | Planejamento revisto, não aprovado para implementação.  
> Entradas: [IDEIA.md](IDEIA.md), [DESIGN.md](DESIGN.md).  
> O método DARE completo será aplicado a cada incremento: Design → Blueprint → Review → Tasks/DAG → Execute → Proof. Este documento define pacotes de entrega, não substitui tasks de execução.

## Linha de base e decisões já consolidadas

- **Ciclo 012 / PR #13: IMPLEMENTADO e mergeado** — Verified Task Contracts & Handoffs. Não reimplementar.
- Rust é fonte de verdade no `dare-cli`; npm antigo não integra o desenho.
- `dare-agent-coding-run-time` é o runtime nativo atual; `dare-agent-run-time` é legado e não deve ser integrado.
- `dare-harness` instala regras/skills nas IDEs; não confundir com algoritmo Harness².
- `dare-evolution` e `dare-telemetry` pertencem ao CLI (se optimizer open source); runtime próprio atua como alvo/executor nativo opcional.
- PR #14 aberto remove `DARE/` do repositório público; decisões e documentos públicos em `docs/`. Artefatos de ciclo de execução locais continuam sob `DARE/cycles/` quando forem preparados com a metodologia.

## Incrementos/PRs propostos (isolados e com checkpoints)

| Ordem | PR lógico | Objetivo | Dependência | Aceite principal |
|---|---|---|---|---|
| P0a | Observability Contract & Local Ledger | Contratos estruturados de uso, correlação e append-only/redação; integra dashboard read-only | Rust CLI atual, 012 | Consumo desconhecido ≠ zero; replay idempotente; dados só locais |
| P0b | IDE Execution Usage Adapters | Codex, Claude, Cursor e Antigravity programáticos + hooks documentados opt-in | P0a | Evidência de provider/CLI identificada e atribuição por task somente quando confiável |
| P0c | Context Efficiency Baseline | Inventário de tamanho de regras, skills, MCP, prompt caching reportado por provedor, perfil por task | P0a/P0b | Baseline comparável; nenhuma alegação de economia sem amostra |
| P1a | Evolution Contracts & Probe Runner | Crate `dare-evolution`, SurfaceSpec allowlist, baseline/candidatos, executor adapters, budgets e worktrees | P0a, `dare-verify` | Mesma fixture, gates comprovados, sem writes em produção |
| P1b | Harness² Propose/Probe/Compose | Geração experimental de variantes, seleção conservadora, composição sem regressão, holdout | P1a/P0c | Proposta de patch e relatório sem autopromoção |
| P1c | Engineering Experience Library | Experiências versionadas vinculadas a stack/modelo/fixture/gates e histórico de regressões | P1b; integra ideia PR4 Infer-forge | Pesquisa e aposentadoria de skills com evidência; sem duplicação de engines |
| P2a | Native Runtime Telemetry Adapter | Integração versionada com JSONL do `dare-agent-coding-run-time`; cache Anthropic separável na fonte | P0a; versão negociada com runtime | Execução na Cloud correlacionada a task; cache-read/write discriminados |
| P2b | Native Runtime Experiment Adapter | Testar Runtime Pack/config permitidas em sandbox, sem afrouxar segurança | P1b/P2a | Seleção isolada, teste de limite e rollback |
| P3 (opcional) | MCP Native Task Delegation | IDE delega task inteira ao runtime com contrato, gates e recibos | P2; demanda validada | Sem ciclo de agentes duplicado; autorização e audit trail |

**Importante:** P2a pode ser antecipado em paralelo aos experimentos; integrações com o runtime privado não bloqueiam a usabilidade do CLI público. P3 não integra o MVP.

## Trabalhos Infer-forge existentes (não duplicar)

| Trilha lógica | Plano revisado |
|---|---|
| PR1 Verified Task Contracts & Handoffs | Fechado (ciclo 012 / PR #13). Reutilizar |
| PR2 Durable Loop Blocks & Recovery | Feature própria sobre `dare-agent/dare-dag`; não requisito do Harness² MVP |
| PR3 Adaptive Task Graph | Feature própria com alterações do DAG sempre sujeitas a revisão |
| PR4 Engineering Journal & Capability Evolution | Reaproveitar em P1c com ownership da Experience Library no `dare-evolution` |

Quando PR2/PR3 forem retomados, reaproveitar `dare-telemetry` e `dare-verify`, mas não acoplar o motor de evolução à execução de tarefas normais.

## Critérios de entrada/saída por estágio

**Entrada P0:** cobertura real dos adaptadores, outputs amostrados anonimizados, decisão sobre opt-in e retenção. **Saída P0:** schemas, testes e relatório baseline, inclusive `unavailable/unattributed` quando aplicável.

**Entrada P1:** fixtures representativas, limites de orçamento, SurfaceSpec imutável, fonte de resultados normalizada. **Saída P1:** recomposição comparável, provas de gate, geração de PR de proposta sem apply, holdout e variações repetidas quando possível.

**Entrada P2:** protocolo/versão do runtime negociado, policies do Runtime Pack revisadas, benchmark em sandbox. **Saída P2:** mesmo contrato de telemetria, custos/cache corretos, sem alterações de permissão nem execução em produção.

**Promoção:** melhoria não pode passar se falsificar evidências, falhar gates, depender de métricas não reportadas para provar ganho, ou alterar defaults sem aprovação humana.

## Organização dos futuros artefatos DARE de execução

Após aprovação do Design e abertura do ciclo local:

```text
DARE/cycles/rust-harness-evolution-013/
  IDEIA.md
  DESIGN.md
  BLUEPRINT.md
  REVIEW.md
  APPROVAL.md
  TASKS.md
  dare-dag.yaml
  EXECUTION/task-*.md
  REGRESSION.md
  PROOF.md
```

Não gerar um DAG de tasks nem alterar a `main` antes da revisão do Design. Se a numeração 013 passar a estar ocupada, resolver novo identificador no momento de criação sem sobrescrever ciclos existentes.

## Próximos passos concretos

1. Revisão técnica deste DESIGN (incluindo disponibilidade real das APIs e protocolos por IDE).
2. Decisões registradas em APPROVAL pelo responsável humano, especialmente open source versus engine privada e nível de precisão do MVP.
3. Blueprint com contratos, interfaces Rust, storage e matrizes de compatibilidade.
4. Task decomposition e DAG com PR por incremento; CI/gates e relatório de regressão em cada PR.
5. Nenhum merge, ativação de coleta ou deploy sem autorização distinta.
