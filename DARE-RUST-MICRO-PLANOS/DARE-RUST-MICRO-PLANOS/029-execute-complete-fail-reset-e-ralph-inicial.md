# 029 — Execute: complete, fail, reset e Ralph inicial

**Objetivo:** Adicionar transicoes de tarefa e gates build-test-lint.

**Posicao na sequencia:** 29 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **execute: complete, fail, reset e ralph inicial**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 006, 026 e 028 concluidos

## Escopo incluido

- [ ] Implementar --complete, --fail e --reset
- [ ] Criar adapters de gates por stack
- [ ] Executar build, test e lint
- [ ] Aplicar timeout 600s
- [ ] Bloquear DONE em gate falho
- [ ] Registrar attempts e outputs
- [ ] Ingestao basica pos-DONE

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-verify/src/ralph.rs`
- `crates/dare-cli/src/commands/execute.rs`

## Comandos ou superficies afetadas

- `dare execute --complete <id>`
- `dare execute --fail <id>`
- `dare execute --reset <id>`

## Contratos de disco afetados

- `.dare/state.json`
- `.dare/verification/**`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar --complete, --fail e --reset.
- [ ] Implementar e revisar: Criar adapters de gates por stack.
- [ ] Implementar e revisar: Executar build, test e lint.
- [ ] Implementar e revisar: Aplicar timeout 600s.
- [ ] Implementar e revisar: Bloquear DONE em gate falho.
- [ ] Implementar e revisar: Registrar attempts e outputs.
- [ ] Implementar e revisar: Ingestao basica pos-DONE.

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

- [ ] DONE exige gates aprovados.
- [ ] Timeout retorna 124.
- [ ] Reset preserva historico conforme contrato.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Execute: complete, fail, reset e Ralph inicial**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`030-execute-agent-mock-worktrees-e-budget.md`](030-execute-agent-mock-worktrees-e-budget.md).
