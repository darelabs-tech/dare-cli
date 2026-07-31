# 041 — GraphRAG: ingest, keyword, BFS e RRF

**Objetivo:** Entregar busca hibrida basica sem obrigar download de modelo.

**Posicao na sequencia:** 41 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **graphrag: ingest, keyword, bfs e rrf**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 35 e 40 concluidos

## Escopo incluido

- [ ] Indexar arquivos por contentHash
- [ ] Indexar simbolos por regex inicialmente
- [ ] Implementar keyword search com LIKE/FTS5 conforme ADR
- [ ] Implementar BFS 2 hops
- [ ] Implementar RRF k=60
- [ ] Criar ingest/query/stats/viz
- [ ] Limitar traverse maxHops e fanout
- [ ] Adicionar golden rankings

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-graph/src/ingest.rs`
- `crates/dare-graph/src/search.rs`
- `crates/dare-cli/src/commands/graph.rs`

## Comandos ou superficies afetadas

- `dare graph ingest`
- `dare graph query`
- `dare graph stats`
- `dare graph viz`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Indexar arquivos por contentHash.
- [ ] Implementar e revisar: Indexar simbolos por regex inicialmente.
- [ ] Implementar e revisar: Implementar keyword search com LIKE/FTS5 conforme ADR.
- [ ] Implementar e revisar: Implementar BFS 2 hops.
- [ ] Implementar e revisar: Implementar RRF k=60.
- [ ] Implementar e revisar: Criar ingest/query/stats/viz.
- [ ] Implementar e revisar: Limitar traverse maxHops e fanout.
- [ ] Implementar e revisar: Adicionar golden rankings.

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

- [ ] Funciona sem modelo semantico.
- [ ] Ranking e deterministico.
- [ ] Reindexacao sem mudanca e incremental.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **GraphRAG: ingest, keyword, BFS e RRF**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`042-graphrag-semantico-opcional.md`](042-graphrag-semantico-opcional.md).
