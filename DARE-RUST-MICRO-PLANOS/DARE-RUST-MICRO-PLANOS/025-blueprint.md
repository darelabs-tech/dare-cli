# 025 — Blueprint

**Objetivo:** Gerar blueprint, tasks, DAG e execution specs a partir do design.

**Posicao na sequencia:** 25 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **blueprint**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 020, 023 e 024 concluidos

## Escopo incluido

- [ ] Ler DESIGN.md ou path informado
- [ ] Gerar BLUEPRINT.md
- [ ] Gerar TASKS.md
- [ ] Gerar dare-dag.yaml valido
- [ ] Criar DARE/EXECUTION
- [ ] Implementar --force
- [ ] Validar DAG apos geracao
- [ ] Criar capability dare-blueprint

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-cli/src/commands/blueprint.rs`
- `assets/capabilities/dare-blueprint`

## Comandos ou superficies afetadas

- `dare blueprint`
- `dare blueprint <design>`
- `dare blueprint --force`

## Contratos de disco afetados

- `DARE/BLUEPRINT.md`
- `DARE/TASKS.md`
- `DARE/dare-dag.yaml`
- `DARE/EXECUTION/**`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Ler DESIGN.md ou path informado.
- [ ] Implementar e revisar: Gerar BLUEPRINT.md.
- [ ] Implementar e revisar: Gerar TASKS.md.
- [ ] Implementar e revisar: Gerar dare-dag.yaml valido.
- [ ] Implementar e revisar: Criar DARE/EXECUTION.
- [ ] Implementar e revisar: Implementar --force.
- [ ] Implementar e revisar: Validar DAG apos geracao.
- [ ] Implementar e revisar: Criar capability dare-blueprint.

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

- [ ] Artefatos passam dare validate.
- [ ] Sem --force nao sobrescreve customizacoes.
- [ ] Outputs sao deterministas.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Blueprint**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`026-dag-parser-ranks-e-state-store.md`](026-dag-parser-ranks-e-state-store.md).
