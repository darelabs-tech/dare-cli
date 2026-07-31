# 019 — Discover: instalacao do DARE

**Objetivo:** Transformar o detection report em uma instalacao idempotente para os quatro harnesses.

**Posicao na sequencia:** 19 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **discover: instalacao do dare**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 011 a 014 e 018 concluidos

## Escopo incluido

- [ ] Criar InstallPlan
- [ ] Gerar dare.config.json
- [ ] Criar DARE e .dare
- [ ] Materializar templates e dare-graph.yml
- [ ] Mesclar .gitignore
- [ ] Aplicar adapters dos harnesses
- [ ] Implementar rollback em falha
- [ ] Adicionar capability dare-discover

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-project/src/install.rs`
- `assets/capabilities/dare-discover`

## Comandos ou superficies afetadas

- `dare discover`

## Contratos de disco afetados

- `dare.config.json`
- `DARE/**`
- `.dare/**`
- `templates/**`
- `dare-graph.yml`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Criar InstallPlan.
- [ ] Implementar e revisar: Gerar dare.config.json.
- [ ] Implementar e revisar: Criar DARE e .dare.
- [ ] Implementar e revisar: Materializar templates e dare-graph.yml.
- [ ] Implementar e revisar: Mesclar .gitignore.
- [ ] Implementar e revisar: Aplicar adapters dos harnesses.
- [ ] Implementar e revisar: Implementar rollback em falha.
- [ ] Implementar e revisar: Adicionar capability dare-discover.

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

- [ ] Executar duas vezes nao duplica arquivos.
- [ ] Falha restaura estado anterior.
- [ ] Todos os harnesses validam.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Discover: instalacao do DARE**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`020-validate.md`](020-validate.md).
