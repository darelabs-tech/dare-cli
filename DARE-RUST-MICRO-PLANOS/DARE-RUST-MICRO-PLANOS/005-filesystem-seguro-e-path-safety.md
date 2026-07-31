# 005 — Filesystem seguro e path safety

**Objetivo:** Criar primitivas seguras para leitura, escrita, backup e validacao de paths.

**Posicao na sequencia:** 5 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **filesystem seguro e path safety**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 002 e 004 concluidos

## Escopo incluido

- [ ] Implementar ProjectRoot e SafeRelativePath
- [ ] Bloquear path traversal
- [ ] Tratar symlinks e junctions
- [ ] Implementar escrita atomica com fsync quando aplicavel
- [ ] Criar backup e restore
- [ ] Normalizar paths internos para barra POSIX
- [ ] Cobrir drive letters e UNC no Windows
- [ ] Implementar file locks

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-core/src/fs`
- `crates/dare-core/src/path.rs`

## Contratos de disco afetados

- `dare.config.json`
- `.dare/**`
- `DARE/**`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar ProjectRoot e SafeRelativePath.
- [ ] Implementar e revisar: Bloquear path traversal.
- [ ] Implementar e revisar: Tratar symlinks e junctions.
- [ ] Implementar e revisar: Implementar escrita atomica com fsync quando aplicavel.
- [ ] Implementar e revisar: Criar backup e restore.
- [ ] Implementar e revisar: Normalizar paths internos para barra POSIX.
- [ ] Implementar e revisar: Cobrir drive letters e UNC no Windows.
- [ ] Implementar e revisar: Implementar file locks.

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

- [ ] Tentativas de escape falham com erro explicito.
- [ ] Escrita interrompida nao corrompe arquivo anterior.
- [ ] Testes passam em Windows e Unix.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Filesystem seguro e path safety**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`006-execucao-segura-de-processos.md`](006-execucao-segura-de-processos.md).
