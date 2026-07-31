# 033 — Refine e sub-DAG

**Objetivo:** Portar avaliacao de complexidade, split e spliceSubDag.

**Posicao na sequencia:** 33 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **refine e sub-dag**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 020, 026 e 032 concluidos

## Escopo incluido

- [ ] Calcular LOW/MED/HIGH/CRITICAL
- [ ] Gerar proposta de split
- [ ] Implementar --apply
- [ ] Criar spliceSubDag
- [ ] Limitar profundidade a 2
- [ ] Bloquear ciclos
- [ ] Preservar parentId e dependsOn
- [ ] Aplicar exit code 2 em strict

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-dag/src/subdag.rs`
- `crates/dare-cli/src/commands/refine.rs`
- `assets/capabilities/dare-refine`

## Comandos ou superficies afetadas

- `dare refine <task-id>`

## Contratos de disco afetados

- `DARE/dare-dag.yaml`
- `.dare/state.json`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Calcular LOW/MED/HIGH/CRITICAL.
- [ ] Implementar e revisar: Gerar proposta de split.
- [ ] Implementar e revisar: Implementar --apply.
- [ ] Implementar e revisar: Criar spliceSubDag.
- [ ] Implementar e revisar: Limitar profundidade a 2.
- [ ] Implementar e revisar: Bloquear ciclos.
- [ ] Implementar e revisar: Preservar parentId e dependsOn.
- [ ] Implementar e revisar: Aplicar exit code 2 em strict.

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

- [ ] Split aplicado produz DAG valido.
- [ ] Cycle e MaxDepth geram erros especificos.
- [ ] Strict HIGH/CRITICAL retorna 2.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Refine e sub-DAG**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`034-guard.md`](034-guard.md).
