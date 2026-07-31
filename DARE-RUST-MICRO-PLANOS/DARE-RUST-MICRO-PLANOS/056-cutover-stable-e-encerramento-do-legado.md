# 056 — Cutover, stable e encerramento do legado

**Objetivo:** Tornar Rust a implementacao oficial e conduzir a retirada controlada do npm legado.

**Posicao na sequencia:** 56 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **cutover, stable e encerramento do legado**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 055 concluido
- [ ] Pre-requisitos globais aprovados

## Escopo incluido

- [ ] Publicar v1.0.0 stable
- [ ] Atualizar documentacao e instaladores
- [ ] Tornar Rust recomendado
- [ ] Mover npm TypeScript para legacy
- [ ] Publicar janela de suporte
- [ ] Monitorar incidentes
- [ ] Executar plano de rollback se necessario
- [ ] Arquivar componentes legados conforme politica
- [ ] Publicar relatorio final de compatibilidade

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `docs/migration`
- `docs/support`
- `CHANGELOG.md`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Publicar v1.0.0 stable.
- [ ] Implementar e revisar: Atualizar documentacao e instaladores.
- [ ] Implementar e revisar: Tornar Rust recomendado.
- [ ] Implementar e revisar: Mover npm TypeScript para legacy.
- [ ] Implementar e revisar: Publicar janela de suporte.
- [ ] Implementar e revisar: Monitorar incidentes.
- [ ] Implementar e revisar: Executar plano de rollback se necessario.
- [ ] Implementar e revisar: Arquivar componentes legados conforme politica.
- [ ] Implementar e revisar: Publicar relatorio final de compatibilidade.

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

- [ ] Rust e canal oficial.
- [ ] Instalacao nao exige Node/npm.
- [ ] Politica do legado esta publicada.
- [ ] Metricas e incident response ativos.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Cutover, stable e encerramento do legado**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Encerramento

Este e o ultimo microplano da sequencia. O trabalho passa para operacao, manutencao, seguranca e evolucao sem dependencia do legado TypeScript.
