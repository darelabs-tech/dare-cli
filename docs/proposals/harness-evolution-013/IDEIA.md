# IDEIA — DARE Harness Evolution (Rust, revisão arquitetural)

> Ciclo proposto: `rust-harness-evolution-013`  
> Data: 2026-10-22  
> Status: PROPOSTA — não aprovada para Blueprint/Tasks/Execute  
> Repositório de referência: `darelabs-tech/dare-cli` (Rust v4; `main`)  
> Origem: Google Research Harness², LMSYS Infer-forge, arquitetura atual do DARE e decisões da conversa de 2026-10-22.

## 1. Problema e motivação

O DARE é **um método independente do harness executor**. O `dare-cli` fornece regras, skills, comandos, contratos, DAG, gates, integração MCP e adaptadores. Em Cursor, Codex, Claude Code e Antigravity, a ferramenta hospedeira controla seu próprio agente, inferência e loop. No DARE Cloud, o `dare-agent-coding-run-time` é o harness Rust nativo, validado e utilizado em produção; ele substitui a dependência de harnesses de terceiros.

Hoje não há uma camada única que correlacione qualidade, duração, consumo de tokens, cache e custo **por tarefa e tentativa** entre todos esses modos; tampouco um motor que experimente, compare e promova melhorias em regras/skills/configurações usando evidência reproduzível.

O objetivo NÃO é colocar o runtime nativo entre o Cursor e o método DARE e NÃO é portar o framework Python do Google. É adicionar **DARE Evolution** como capacidade transversal do `dare-cli`, usando um loop de experimentos separado do loop de trabalho.

## 2. Hipótese de solução

**Três planos independentes:**
1. **Método:** `dare-cli` Rust define invariantes, contratos, contexto controlado, DAG, verificação e governança.
2. **Execução:** harnesses de terceiros nas IDEs; `dare-agent-coding-run-time` no DARE Cloud e opcionalmente como executor delegado.
3. **Evolução:** novo `dare-evolution` (Harness² adaptado em Rust) propõe configurações candidatas, executa probes isolados, compara qualidade/custo/segurança, compõe apenas mudanças comprovadas e solicita aprovação humana para promoção.

**Pré-requisito transversal:** `dare-telemetry` reúne metadados com proveniência distinta por executor. A ausência de métricas nunca equivale a zero consumo. As ferramentas externas não cedem automaticamente controle da inferência, do prompt cache ou do loop interno.

## 3. Alvos do mecanismo de evolução

| Alvo | Superfícies alteráveis em experimentos | Limites |
|---|---|---|
| IDEs externas (Cursor, Codex, Claude, Antigravity) | Regras, skills, comandos e respostas MCP **gerenciadas pelo DARE** | Sem modificar internals da IDE/harness, sem garantir cache ou contagem precisa fora das interfaces documentadas |
| Runtime nativo DARE | Runtime Pack, composição de contexto, hints de ferramentas e outras configurações explicitamente expostas | Sem relaxar políticas, sem alterar a instância em produção; protocolo versionado e sandbox isolada |
| Método DARE | Sugestões para templates, seleção de contexto e experiência entre tarefas | Nunca alterar invariantes, gates ou contratos aprovados silenciosamente |

## 4. Critérios de sucesso

- Relatório reproduzível por `taskId/runId/attemptId/executor/model`, com campos de tokens, cache, custo, tempo, gates e fonte/qualidade dos dados.
- Baseline e candidatos avaliados nas **mesmas** fixtures de tarefa; rodadas repetidas quando houver não determinismo.
- Melhorias só promovidas se preservarem os gates de qualidade, segurança e compatibilidade e apresentarem benefício mensurável.
- `dare evolve` desativado por padrão, opt-in explícito, limites de orçamento e artefatos de auditoria.
- Uma proposta de melhoria gera diff e evidências para aprovação humana antes de atualizar arquivos gerenciados ou qualquer runtime de produção.
- `dare-cli` continua utilizável sem o runtime privado nem dependências Python.

## 5. Não objetivos

- Não substituir Cursor/Codex/Claude/Antigravity como harness.
- Não usar o repositório legado `dare-agent-run-time`; somente `dare-agent-coding-run-time` é referência nativa.
- Não inserir o runtime entre a IDE e as chamadas de modelo para capturar prompts.
- Não prometer redução de tokens de contexto pelo caching: Prompt Caching altera custo de processamento, não quantidade lógica de contexto.
- Não implementar engine de inferência, execução distribuída ou modelos novos dentro do CLI.
- Não ativar coleta remota, executar benchmarks pagos ou fazer merge/deploy automaticamente.

## 6. Relação com os quatro trabalhos derivados do Infer-forge

- **PR lógico 1 — Verified Task Contracts & Handoffs:** ciclo 012, implementado via PR #13. Reutilizar os contratos/recibos existentes; não reabrir.
- **PR lógico 2 — Durable Loop Blocks & Recovery:** trilha de execução confiável distinta. Pode fornecer evidências mais ricas ao Evolution posteriormente.
- **PR lógico 3 — Adaptive Task Graph:** propostas de mudanças do DAG somente mediante revisão humana; não confundir evolução de DAG com evolução de skills.
- **PR lógico 4 — Engineering Journal & Capability Evolution:** parte da biblioteca de experiência deve integrar o `dare-evolution`; evitar duas bibliotecas ou mecanismos de promoção concorrentes.

A dependência obrigatória para o MVP do Harness² é **telemetria + avaliação reproduzível**, não concluir PR2/PR3 inteiros.

## 7. Entregáveis de planejamento

- [DESIGN.md](DESIGN.md): escopo, requisitos, isolamento, segurança, critérios e interfaces propostas.
- [ROADMAP.md](ROADMAP.md): pacotes/PRs e dependências, mantendo o método DARE completo.
- Após revisão e aprovação explícita: gerar o BLUEPRINT, TASKS, DAG YAML e specs de execução por tarefa no ciclo local `DARE/cycles/rust-harness-evolution-013/`.

**Observação sobre versionamento:** a pasta `DARE/` está ignorada no repositório público e o PR #14 propõe removê-la do rastreamento Git. Este registro de proposta vive em `docs/proposals/` para não contrariar essa separação. A aprovação humana não é inferida da abertura de um PR de documentação.

## 8. Fontes

- Google Research: https://github.com/google-research/harness2
- LMSYS, Infer-forge: https://www.lmsys.org/blog/2026-08-28-infer-forge-loop-engineering/
- DARE CLI: https://github.com/darelabs-tech/dare-cli
- Runtime nativo atual: https://github.com/darelabs-tech/dare-agent-coding-run-time
