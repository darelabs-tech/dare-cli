# 030 — Execute agent: mock, worktrees e budget

**Objetivo:** Validar a maquina autonoma sem variabilidade de agentes reais.

**Posicao na sequencia:** 30 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **execute agent: mock, worktrees e budget**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 006, 029 e 034 planejado para preflight

## Escopo incluido

- [ ] Definir AgentDriver
- [ ] Implementar mock/noop
- [ ] Criar worktrees e branches
- [ ] Implementar BudgetTracker
- [ ] Adicionar cancellation token
- [ ] Registrar failureSignature
- [ ] Implementar politica fixed
- [ ] Criar limpeza e recovery de worktrees

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-agent`
- `crates/dare-cli/src/commands/execute_agent.rs`

## Comandos ou superficies afetadas

- `dare execute --agent --driver mock`

## Contratos de disco afetados

- `.dare/agent-worktrees/**`
- `.dare/state.json`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir AgentDriver.
- [ ] Implementar e revisar: Implementar mock/noop.
- [ ] Implementar e revisar: Criar worktrees e branches.
- [ ] Implementar e revisar: Implementar BudgetTracker.
- [ ] Implementar e revisar: Adicionar cancellation token.
- [ ] Implementar e revisar: Registrar failureSignature.
- [ ] Implementar e revisar: Implementar politica fixed.
- [ ] Implementar e revisar: Criar limpeza e recovery de worktrees.

### 3. Compatibilidade

- [ ] Comparar comportamento observavel com a versao TypeScript 3.18.1.
- [ ] Registrar diferencas intencionais no changelog e ADR correspondente.
- [ ] Preservar paths, IDs, formatos e ordenacoes deterministicas aplicaveis.
- [ ] Confirmar funcionamento em Linux, macOS e Windows quando houver I/O, processos ou paths.

### 4. Seguranca

- [ ] Validar entradas e limites.
- [ ] Aplicar path safety em toda leitura e escrita.
- [ ] Evitar shell concatenado; usar argv separado para processos.
- [ ] Redigir secrets, tokens e dados sensiveis de logs e erros.
- [ ] Testar falhas parciais, cancelamento e rollback quando aplicavel.

### 5. Documentacao e release

- [ ] Atualizar help, documentacao tecnica e exemplos.
- [ ] Atualizar matriz de compatibilidade/capabilities quando aplicavel.
- [ ] Adicionar release notes.
- [ ] Gerar binarios, checksums e smoke tests no canal atual.

## Estrategia de testes

- [ ] Testes unitarios do dominio.
- [ ] Testes de integracao com filesystem/processos reais quando aplicavel.
- [ ] Golden tests contra a implementacao TypeScript.
- [ ] Casos de erro e entradas malformadas.
- [ ] Testes cross-platform para comportamento dependente de sistema.
- [ ] Testes de seguranca relevantes ao escopo.

## Criterios de aceite

- [ ] Suite mock cobre sucesso/falha/timeout.
- [ ] Orcamento interrompe execucao.
- [ ] Worktrees sao limpas ou recuperaveis.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Execute agent: mock, worktrees e budget**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`031-drivers-reais-de-agentes.md`](031-drivers-reais-de-agentes.md).
