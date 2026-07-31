# 043 — GraphRAG avancado e Neo4j

**Objetivo:** Portar locate, impact, owners, trace, drift e backend experimental.

**Posicao na sequencia:** 43 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **graphrag avancado e neo4j**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 040 a 042 concluidos

## Escopo incluido

- [ ] Implementar locate com decay
- [ ] Implementar owners
- [ ] Implementar impact
- [ ] Implementar trace
- [ ] Implementar drift orphan/stale
- [ ] Aplicar threshold e exit code 7
- [ ] Adicionar Neo4j HTTP experimental
- [ ] Criar limites de timeout e retries

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-graph/src/advanced.rs`
- `crates/dare-graph/src/neo4j.rs`

## Comandos ou superficies afetadas

- `dare graph locate`
- `dare graph owners`
- `dare graph impact`
- `dare graph trace`
- `dare graph drift`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar locate com decay.
- [ ] Implementar e revisar: Implementar owners.
- [ ] Implementar e revisar: Implementar impact.
- [ ] Implementar e revisar: Implementar trace.
- [ ] Implementar e revisar: Implementar drift orphan/stale.
- [ ] Implementar e revisar: Aplicar threshold e exit code 7.
- [ ] Implementar e revisar: Adicionar Neo4j HTTP experimental.
- [ ] Implementar e revisar: Criar limites de timeout e retries.

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

- [ ] Drift strict retorna 7.
- [ ] Traverse respeita limites.
- [ ] Neo4j fica opt-in experimental.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **GraphRAG avancado e Neo4j**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`044-skills-registry-modelo-e-resolucao.md`](044-skills-registry-modelo-e-resolucao.md).
