# 032 — Review

**Objetivo:** Portar analise estatica e formatos de resultado do dare review.

**Posicao na sequencia:** 32 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **review**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 024, 025 e 029 concluidos

## Escopo incluido

- [ ] Detectar stubs, mocks e TODOs
- [ ] Definir severidades
- [ ] Implementar --strict e --errors-only
- [ ] Implementar --files e --from-agent
- [ ] Gerar human/json/github
- [ ] Implementar --comment e --fail-on
- [ ] Adicionar enrichment opcional
- [ ] Criar capability dare-review

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-review`
- `crates/dare-cli/src/commands/review.rs`
- `assets/capabilities/dare-review`

## Comandos ou superficies afetadas

- `dare review <task-id>`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Detectar stubs, mocks e TODOs.
- [ ] Implementar e revisar: Definir severidades.
- [ ] Implementar e revisar: Implementar --strict e --errors-only.
- [ ] Implementar e revisar: Implementar --files e --from-agent.
- [ ] Implementar e revisar: Gerar human/json/github.
- [ ] Implementar e revisar: Implementar --comment e --fail-on.
- [ ] Implementar e revisar: Adicionar enrichment opcional.
- [ ] Implementar e revisar: Criar capability dare-review.

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

- [ ] Exit codes correspondem ao fail-on.
- [ ] Formato GitHub e valido.
- [ ] Resultados estaticos sao deterministas.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Review**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`033-refine-e-sub-dag.md`](033-refine-e-sub-dag.md).
