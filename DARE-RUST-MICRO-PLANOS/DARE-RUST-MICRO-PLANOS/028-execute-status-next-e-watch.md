# 028 — Execute: status, next e watch

**Objetivo:** Entregar navegacao e observacao deterministicas do DAG.

**Posicao na sequencia:** 28 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **execute: status, next e watch**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 026 concluido

## Escopo incluido

- [ ] Implementar --status default
- [ ] Implementar --next
- [ ] Compor prompt com parent context cap
- [ ] Implementar --watch
- [ ] Atualizar canvas
- [ ] Adicionar saida JSON
- [ ] Tratar DAG vazio e bloqueado

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-cli/src/commands/execute.rs`
- `crates/dare-dag/src/execution.rs`

## Comandos ou superficies afetadas

- `dare execute --status`
- `dare execute --next`
- `dare execute --watch`

## Contratos de disco afetados

- `.dare/state.json`
- `DARE/.canvas.md`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar --status default.
- [ ] Implementar e revisar: Implementar --next.
- [ ] Implementar e revisar: Compor prompt com parent context cap.
- [ ] Implementar e revisar: Implementar --watch.
- [ ] Implementar e revisar: Atualizar canvas.
- [ ] Implementar e revisar: Adicionar saida JSON.
- [ ] Implementar e revisar: Tratar DAG vazio e bloqueado.

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

- [ ] Next retorna somente menor rank executavel.
- [ ] Parent context respeita limite.
- [ ] Watch nao altera estado.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Execute: status, next e watch**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`029-execute-complete-fail-reset-e-ralph-inicial.md`](029-execute-complete-fail-reset-e-ralph-inicial.md).
