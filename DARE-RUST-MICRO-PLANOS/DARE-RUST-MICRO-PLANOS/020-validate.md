# 020 — Validate

**Objetivo:** Portar a validacao completa do DAG com saida deterministica.

**Posicao na sequencia:** 20 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **validate**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 004, 007 e 008 concluidos

## Escopo incluido

- [ ] Parser v2.1 e legado
- [ ] Validar IDs unicos e kebab-case
- [ ] Validar dependencias e referencias
- [ ] Detectar ciclos
- [ ] Validar prompts/specs
- [ ] Implementar --strict
- [ ] Mapear exit codes
- [ ] Adicionar human e JSON

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-dag/src/validate.rs`
- `crates/dare-cli/src/commands/validate.rs`

## Comandos ou superficies afetadas

- `dare validate`
- `dare validate --strict`
- `dare validate --dag <path>`

## Contratos de disco afetados

- `DARE/dare-dag.yaml`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Parser v2.1 e legado.
- [ ] Implementar e revisar: Validar IDs unicos e kebab-case.
- [ ] Implementar e revisar: Validar dependencias e referencias.
- [ ] Implementar e revisar: Detectar ciclos.
- [ ] Implementar e revisar: Validar prompts/specs.
- [ ] Implementar e revisar: Implementar --strict.
- [ ] Implementar e revisar: Mapear exit codes.
- [ ] Implementar e revisar: Adicionar human e JSON.

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

- [ ] Resultados equivalem a fixtures TypeScript.
- [ ] Ordenacao dos erros e estavel.
- [ ] Validacao nao escreve.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Validate**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`021-update-planejamento-e-manifest.md`](021-update-planejamento-e-manifest.md).
