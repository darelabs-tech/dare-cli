# DESIGN — DARE Harness Evolution, Telemetry & Runtime Boundaries

> Ciclo proposto: `rust-harness-evolution-013` · 2026-10-22  
> Fase D / Design: **PROPOSTO, sujeito a revisão humana**. Não autoriza Blueprint, implementação ou merge.  
> Referência: [IDEIA.md](IDEIA.md), [ROADMAP.md](ROADMAP.md).

## 1. Contexto e baseline verificado

O DARE CLI v4.0.0 é workspace Rust (MSRV 1.88.0, Apache-2.0), com `dare-harness` (instala artefatos em IDEs), `dare-agent` (drivers Codex/Claude/Cursor/Antigravity), `dare-dag`, `dare-verify` (Fix·Rate/bench e gates), `dare-contracts` (Task Contracts/Handoff + `TelemetrySnapshot`), `dare-server` (dashboard/MCP), `dare-graph` e `dare-skills`. Handoff verificado foi incorporado no PR #13/ciclo 012. O dashboard atual inicializa `cost` vazio. `AgentRunResult.tokens: Option<u64>` não captura entrada/saída/cache; `execute_agent.rs` converte ausência em zero ao debitar budget, fragilidade de observabilidade.

O DARE Agent Coding Runtime atual é o repositório **privado** `darelabs-tech/dare-agent-coding-run-time`, em Rust; `dare-agent-run-time` é legado fora de escopo. Runtime atual suporta `TaskSpec`, `GateFeedback` e eventos JSONL `SessionStarted/TokenUsage/TaskResult`, além de Prompt Caching Anthropic marcando prefixo e última mensagem. A estrutura `Usage` atual combina tokens de cache em `prompt_tokens`; logo não permite precificação diferenciada nem cálculo de cache hit rate. Qualquer mudança em protocolos nativos é um trabalho coordenado, versionado e compatível, **não uma dependência para executar o CLI junto às IDEs**.

No modo IDE, o agente e a inferência pertencem a Cursor/Codex/Claude/Antigravity. A instalação de regras DARE não outorga acesso ao cache interno nem garante métricas reais. Fluxos de uso interativo e `dare execute --agent` são distintos e precisam de rastreadores separados.

## 2. Objetivos e requisitos

| ID | Requisito | Aceite |
|---|---|---|
| RF-01 | Observabilidade de execução agnóstica de harness | Registros correlacionados por projeto/ciclo/task/run/attempt/session/executor/model, com provider opcional |
| RF-02 | Contagem fiel de uso e cache | Campos separados entrada, saída, cache-read, cache-write, fonte e status; valores desconhecidos jamais viram zero |
| RF-03 | IDE adapters não invasivos | Captura só por hooks, CLIs estruturadas, APIs oficiais ou eventos MCP autorizados; sem interceptação oculta |
| RF-04 | Otimização de contexto | Testes de carregamento seletivo de skills, organização estável de prompts DARE e resposta MCP; não prometer economia sem benchmark |
| RF-05 | Harness² em Rust | Motor Propose → Probe → Compose e biblioteca de experiências; baseline conservado e candidatos isolados |
| RF-06 | Protocolo de execução desacoplado | `ExecutionAdapter` que aceita fixtures/tarefas, retorna provas e métricas; adaptadores IDE/Native independentes |
| RF-07 | Promoção governada | Regressão, segurança, integridade e holdout obrigatórios; diff revisável; nenhum apply automático |
| RF-08 | Runtime nativo como alvo opcional | Adaptador para `dare-agent-coding-run-time` via protocolo versionado; execução nativa e delegação não obrigatórias |
| RF-09 | Relatórios locais e dashboard | JSONL append-only por evento e relatórios agregados por tarefa; dashboard read-only sem telemetria remota automática |
| RF-10 | Compatibilidade e segurança | `dare execute`, `dare init/update/harness`, `dare mcp` e `--json` atuais preservados por padrão |

## 3. Arquitetura proposta

```text
  DARE CLI (método, contratos, DAG, skills, handoff, gates, MCP)
       |                                    |
       +-- Execução normal -----------------+------> IDE externa / próprio harness
       |                                    +------> DARE Coding Runtime / DARE Cloud
       |
       +-- Observability adapters -----------------> dare-telemetry (Rust)
       |                                                       |
       +-- Experimentos opt-in --> dare-evolution (Rust) <-----+
                                    Propose / Probe / Compose
                                         |
                                  dare-verify (gates)
                                         |
                                    Proposta + evidência
                                         |
                                  Review humana / PR
```

**Plano de controle:** CLI governa tarefas e aprovação; IDE/runtime executa. **Plano de avaliação:** Evolution experimenta variantes isoladas; não intercepta chamadas normais. **Plano de dados:** Telemetry correlaciona evidências sem coletar por padrão texto de prompts, stdout bruto ou dados pessoais.

### Organização provável de crates

```text
crates/dare-telemetry/     (novo; contratos de uso, normalização, storage, agregação)
crates/dare-evolution/     (novo; surface allowlist, propose, probe, compose, library, score)
crates/dare-cli/src/commands/telemetry.rs    (proposto)
crates/dare-cli/src/commands/evolve.rs       (proposto)
crates/dare-harness/      (adapters opt-in; não um runtime executor)
crates/dare-agent/        (drivers estruturados; adaptação opcional ao runtime nativo)
crates/dare-verify/       (reutilizar gates e bench)
crates/dare-server/       (visualização local)
crates/dare-contracts/    (versionar envelope somente se necessário)
```

**Atenção:** o crate `dare-harness` existente INSTALA integração de IDE; não é Harness². `dare-evolution` fica separado, evitando reinterpretação desse crate.

## 4. Modelo de telemetria

Evento mínimo versionado: `eventId`, `timestamp`, `projectId`, `cycleId?`, `taskId?`, `runId`, `attemptId?`, `sessionId?`, `executor`, `provider?`, `model?`, `phase`, `usage {inputTokens?,outputTokens?,cacheReadTokens?,cacheWriteTokens?,reasoningTokens?}`, `usageSource(provider_reported|cli_reported|estimated|unavailable)`, `durationMs?`, `cost {amount?,currency?,pricingVersion?,isEstimate}`, `verificationRef?`, `outcome`. Para fornecedores com múltiplos formatos, mapear com adaptador explícito e testes de fixtures reais anonimizadas.

- Contagem não disponível: `null`, jamais `0`. `budget.consume(result.tokens.unwrap_or(0))` exige tratamento explícito: estimativa conservadora, execução limitada/aborto, ou estado unmetered identificado por política.
- Cache-read/caching só quando reportado. Fator de preço não é universal; precificação versionada por provider/modelo/data.
- Idempotência por `eventId`/identificador do request e replays; eventos brutos append-only, agregados reconstruíveis.
- Atribuição de uso a task depende de correlação confiável; conversas com múltiplas tasks podem exigir estado `unattributed`; nunca distribuir como medição real sem fonte.
- Arquivos locais sob `.dare/telemetry/`; nenhuma coleta externa sem opt-in. Redação de campos, política de retenção e permissões locais.

## 5. Estratégia Harness²

1. **SurfaceSpec/allowlist:** mutação apenas de regras/skills/templates DARE e parâmetros de runtime explicitamente autorizados; proibir alterações de critérios, gates, verificação, política de segurança, sandbox e orçamento.
2. **Propose:** gerar alternativas de configuração com digests de origem, justificativa e escopo.
3. **Probe:** executar baseline e candidatos com fixtures equivalentes, revisão das versões de modelo/execução, worktrees/sandboxes segregadas e budget rígido; registrar missing metrics.
4. **Compose:** combinar alterações verificadas (sem extrapolar de um teste único); controlar overfitting com fixtures holdout e testes em mais de uma seed/rodada, conforme aplicável.
5. **Experience Library:** `cue`, `context`, `change`, `evidenceRefs`, `models`, `regressions`, `decision`, `revision`; sem acumular recomendações não verificadas como verdade.
6. **Promotion:** apenas proposta de patch/PR após human review; comparar métricas e integridade. Não modificar configurações de produção automaticamente.

**Função objetivo**: priorizar sucesso em gates, correção e segurança; entre alternativas que satisfazem os limites, reduzir custo por task verificada, tokens não cacheados, duração e tentativas. Ausência de dados de custo não pode favorecer um candidato.

### Dois tipos de loop

- **Work Loop:** executa tarefa DARE com IDE externa ou harness nativo; permanece independente do Evolution.
- **Evolution Loop:** experimenta alterações das superfícies DARE permitidas e avalia provas; roda somente sob solicitação explícita.

## 6. Modos de execução e fronteiras

| Caminho | Dono da inferência | Papel do CLI | Telemetria viável | Evolution permitido |
|---|---|---|---|---|
| Cursor interativo | Cursor | Regras/skills/MCP/gates | Hooks + API documentada; pode ser parcial | Variantes das superfícies DARE apenas |
| Codex CLI/IDE | Codex | Regras/skills/MCP/gates e driver | Metadados do CLI se disponíveis | Idem |
| Claude Code | Claude Code | Regras/skills/MCP/gates e driver | JSON/hooks se disponíveis | Idem |
| Antigravity | Antigravity | Regras/skills/MCP/gates | Eventos/hook se disponíveis | Idem |
| DARE Cloud nativo | DARE Coding Runtime | Contratos, gates, telemetria e método | Eventos nativos/provedor | Pode testar Runtime Pack e parâmetros autorizados |

**Delegação do IDE ao runtime nativo:** opção futura via MCP, transação por task com contrato, decisão humana para ações state-changing, política explícita, ambiente isolado e retorno só de evidências, sem subordinar toda sessão do Cursor. Não é pré-requisito P0/P1; não introduzir lógica duplicada de raciocínio.

## 7. Impacto, compatibilidade e controles

Novos crates: `dare-telemetry`, `dare-evolution`. Modificações potenciais: Cargo.toml/lock, `dare-agent` drivers, `dare-harness` installers, `dare-cli` comandos, `dare-server` dashboard, `dare-verify` interfaces, docs. `dare-contracts` somente se exigir contrato compartilhado. Nenhum banco remoto requerido; storage local append-only ou SQLite como decisão do Blueprint. Nenhuma alteração de produção necessária no runtime privado até P2.

Guards: sem exfiltração por MCP; validação de escopo/caminho e assinaturas; blindagem de prompts e logs; limites de CPU/tempo/tokens/$; isolamento cross-branch e por segredo; acesso somente leitura no modo análise; nenhuma criação de agentes privados ou chaves de API. Regressão de custo não pode ser mascarada por missing metrics.

### Restrições de distribuição

O `dare-cli` é Apache-2.0 **público**. Algoritmos colocados no crate `dare-evolution` serão públicos. Caso se deseje algoritmo proprietário exclusivo ao Cloud, o Design deve ser revisto para manter somente o contrato e adaptadores no CLI e implementar o optimizer privado fora dele. Não acoplar obrigatoriamente o CLI ao repositório privado.

## 8. Estratégia de testes e aceitação

- Unit tests: parsers para quatro adapters, `null` versus zero, dedupe, custo versionado, cache-read/write e correlacionamento.
- Integration tests: two-mode (IDE stub/native fake) com fixtures equivalentes, worktrees separados, budgets e replay idempotente.
- Security tests: secrets nos logs, path/symlink escape, acesso MCP, prompt injection em outputs, proibição de mutação fora da allowlist.
- Evolution tests: candidato regressivo rejeitado, holdout, baseline mantido, experimento interrompido sem promoção, aprovação ausente sem apply, verificações por aspecto e evidência persistida em disco.
- Compatibility: golden CLI, help, `--json`, exit codes e instaladores de IDE mantidos.
- CI real requerido ao executar, não neste Design: `cargo fmt --all -- --check`, `cargo build --workspace`, `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo audit`, `cargo deny check` e `dare validate` no DAG aprovado.

## 9. Gate de decisão para a fase A

Ao revisar este Design, decidir somente:
1. O motor `dare-evolution` será integralmente open source no CLI? **Padrão proposto: sim**.
2. O MVP prioriza medição por execuções programáticas, marcando conversas IDE sem correlação como parciais? **Padrão proposto: sim**.
3. Autorizar criação de Blueprint e TASKS após revisão explícita. **Ainda pendente**.

Este documento não declara implementação concluída nem aprova execução ou merge.
