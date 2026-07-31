# 042 — GraphRAG semantico opcional

**Objetivo:** Adicionar embeddings locais sem aumentar a instalacao principal.

**Posicao na sequencia:** 42 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **graphrag semantico opcional**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 041 concluido

## Escopo incluido

- [ ] Definir feature semantic
- [ ] Selecionar fastembed ou ort/tokenizers
- [ ] Usar all-MiniLM-L6-v2 quantizado
- [ ] Baixar sob confirmacao com tamanho exibido
- [ ] Cachear em ~/.dare/models
- [ ] Implementar cosine O(n*d)
- [ ] Fundir ranking vetorial via RRF
- [ ] Fallback automatico keyword+grafo
- [ ] Adicionar comando de enable/doctor se aprovado

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-graph/src/semantic.rs`

## Contratos de disco afetados

- `~/.dare/models/**`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir feature semantic.
- [ ] Implementar e revisar: Selecionar fastembed ou ort/tokenizers.
- [ ] Implementar e revisar: Usar all-MiniLM-L6-v2 quantizado.
- [ ] Implementar e revisar: Baixar sob confirmacao com tamanho exibido.
- [ ] Implementar e revisar: Cachear em ~/.dare/models.
- [ ] Implementar e revisar: Implementar cosine O(n*d).
- [ ] Implementar e revisar: Fundir ranking vetorial via RRF.
- [ ] Implementar e revisar: Fallback automatico keyword+grafo.
- [ ] Implementar e revisar: Adicionar comando de enable/doctor se aprovado.

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

- [ ] CLI base nao inclui modelo.
- [ ] Falha de download nao quebra busca.
- [ ] Modelo e reutilizado entre projetos.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **GraphRAG semantico opcional**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`043-graphrag-avancado-e-neo4j.md`](043-graphrag-avancado-e-neo4j.md).
