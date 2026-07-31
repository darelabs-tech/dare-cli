# 051 — Dashboard e REST compativel

**Objetivo:** Portar dashboard read-only e endpoints HTTP legados em Axum.

**Posicao na sequencia:** 51 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **dashboard e rest compativel**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 017, 026, 40 e 49 concluidos

## Escopo incluido

- [ ] Criar axum app compartilhado
- [ ] Portar dashboard HTML/assets
- [ ] Implementar /api/telemetry
- [ ] Portar health/tools/context/blueprint/dag/tasks/graph/project/steering
- [ ] Aplicar auth, body limit e path safety
- [ ] Bind loopback default
- [ ] Abrir navegador de forma cross-platform
- [ ] Adicionar graceful shutdown

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-server/src/rest.rs`
- `crates/dare-server/src/dashboard.rs`

## Comandos ou superficies afetadas

- `dare dashboard`
- `dare server --protocol rest`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Criar axum app compartilhado.
- [ ] Implementar e revisar: Portar dashboard HTML/assets.
- [ ] Implementar e revisar: Implementar /api/telemetry.
- [ ] Implementar e revisar: Portar health/tools/context/blueprint/dag/tasks/graph/project/steering.
- [ ] Implementar e revisar: Aplicar auth, body limit e path safety.
- [ ] Implementar e revisar: Bind loopback default.
- [ ] Implementar e revisar: Abrir navegador de forma cross-platform.
- [ ] Implementar e revisar: Adicionar graceful shutdown.

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

- [ ] Contract tests HTTP passam.
- [ ] Escape de path retorna 403.
- [ ] Token e obrigatorio fora de loopback.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Dashboard e REST compativel**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`052-mcp-real-como-transporte-separado.md`](052-mcp-real-como-transporte-separado.md).
