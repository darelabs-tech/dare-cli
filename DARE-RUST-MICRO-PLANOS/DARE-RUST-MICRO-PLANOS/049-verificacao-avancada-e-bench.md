# 049 — Verificacao avancada e bench

**Objetivo:** Adicionar aspectos pos-Ralph, mutation, formal, best-of-N e regressao.

**Posicao na sequencia:** 49 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **verificacao avancada e bench**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 029 a 34 concluidos

## Escopo incluido

- [ ] Implementar fail-to-pass
- [ ] Implementar anti-tamper
- [ ] Integrar stryker/mutmut/cargo-mutants/infection
- [ ] Integrar Dafny/Verus/Lean
- [ ] Implementar repair loop
- [ ] Implementar best-of-N e Pareto
- [ ] Implementar decay/replan/escalate
- [ ] Portar bench fixtures e FixRate
- [ ] Adicionar baseline regression

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-verify`
- `crates/dare-cli/src/commands/bench.rs`

## Comandos ou superficies afetadas

- `dare bench`
- `dare execute --best-of <n>`
- `dare execute --full-mutation`
- `dare execute --formal`
- `dare execute --policy decay`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar fail-to-pass.
- [ ] Implementar e revisar: Implementar anti-tamper.
- [ ] Implementar e revisar: Integrar stryker/mutmut/cargo-mutants/infection.
- [ ] Implementar e revisar: Integrar Dafny/Verus/Lean.
- [ ] Implementar e revisar: Implementar repair loop.
- [ ] Implementar e revisar: Implementar best-of-N e Pareto.
- [ ] Implementar e revisar: Implementar decay/replan/escalate.
- [ ] Implementar e revisar: Portar bench fixtures e FixRate.
- [ ] Implementar e revisar: Adicionar baseline regression.

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

- [ ] Regression de pass-to-pass zera FixRate.
- [ ] Mutation threshold aplicado.
- [ ] Formal e opt-in e auditavel.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Verificacao avancada e bench**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`050-comandos-ai.md`](050-comandos-ai.md).
