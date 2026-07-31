# 018 — Discover: deteccao brownfield

**Objetivo:** Implementar analise deterministica de um projeto existente.

**Posicao na sequencia:** 18 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **discover: deteccao brownfield**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 005, 007, 008 e 009 concluidos

## Escopo incluido

- [ ] Localizar project root e Git
- [ ] Detectar stacks por arquivos e manifests
- [ ] Detectar monorepo
- [ ] Detectar harnesses existentes
- [ ] Produzir DetectionReport
- [ ] Implementar --check sem escrita
- [ ] Adicionar fixtures Node, Rust, Python e monorepo

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-project`
- `crates/dare-cli/src/commands/discover.rs`

## Comandos ou superficies afetadas

- `dare discover --check`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Localizar project root e Git.
- [ ] Implementar e revisar: Detectar stacks por arquivos e manifests.
- [ ] Implementar e revisar: Detectar monorepo.
- [ ] Implementar e revisar: Detectar harnesses existentes.
- [ ] Implementar e revisar: Produzir DetectionReport.
- [ ] Implementar e revisar: Implementar --check sem escrita.
- [ ] Implementar e revisar: Adicionar fixtures Node, Rust, Python e monorepo.

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

- [ ] Deteccao e deterministica.
- [ ] Nenhum arquivo e escrito no check.
- [ ] Conflitos de stack sao reportados.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Discover: deteccao brownfield**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`019-discover-instalacao-do-dare.md`](019-discover-instalacao-do-dare.md).
