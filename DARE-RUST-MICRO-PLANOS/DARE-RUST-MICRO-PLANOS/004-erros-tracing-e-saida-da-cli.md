# 004 — Erros, tracing e saida da CLI

**Objetivo:** Padronizar erros, logs, stdout, stderr, cores e modo JSON.

**Posicao na sequencia:** 4 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **erros, tracing e saida da cli**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 002 concluido
- [ ] ADR de JSON e idioma aprovado

## Escopo incluido

- [ ] Definir ErrorKind e exit code mapping
- [ ] Usar thiserror no dominio e anyhow somente na borda
- [ ] Criar OutputRenderer human/json
- [ ] Separar stdout de stderr
- [ ] Controlar ANSI por TTY e NO_COLOR
- [ ] Adicionar tracing com redaction
- [ ] Criar contexto de execucao e correlation id

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-core/src/error.rs`
- `crates/dare-cli/src/output.rs`
- `crates/dare-core/src/telemetry.rs`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir ErrorKind e exit code mapping.
- [ ] Implementar e revisar: Usar thiserror no dominio e anyhow somente na borda.
- [ ] Implementar e revisar: Criar OutputRenderer human/json.
- [ ] Implementar e revisar: Separar stdout de stderr.
- [ ] Implementar e revisar: Controlar ANSI por TTY e NO_COLOR.
- [ ] Implementar e revisar: Adicionar tracing com redaction.
- [ ] Implementar e revisar: Criar contexto de execucao e correlation id.

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

- [ ] Mesmo erro produz exit code deterministico.
- [ ] JSON nao contem mensagens ANSI.
- [ ] Secrets conhecidos sao redigidos.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Erros, tracing e saida da CLI**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`005-filesystem-seguro-e-path-safety.md`](005-filesystem-seguro-e-path-safety.md).
