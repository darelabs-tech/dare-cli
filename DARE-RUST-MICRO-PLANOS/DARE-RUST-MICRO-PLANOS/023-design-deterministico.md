# 023 — Design deterministico

**Objetivo:** Portar a geracao basica de DESIGN.md sem depender de IA.

**Posicao na sequencia:** 23 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **design deterministico**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 009, 010 e 019 concluidos

## Escopo incluido

- [ ] Definir schema de entrada
- [ ] Gerar estrutura canonica de DESIGN.md
- [ ] Implementar modo interativo
- [ ] Preservar areas personalizadas
- [ ] Adicionar markers de enrichment
- [ ] Criar capability dare-design
- [ ] Adicionar snapshots

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-cli/src/commands/design.rs`
- `assets/capabilities/dare-design`

## Comandos ou superficies afetadas

- `dare design <descricao>`
- `dare design --interactive`

## Contratos de disco afetados

- `DARE/DESIGN.md`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir schema de entrada.
- [ ] Implementar e revisar: Gerar estrutura canonica de DESIGN.md.
- [ ] Implementar e revisar: Implementar modo interativo.
- [ ] Implementar e revisar: Preservar areas personalizadas.
- [ ] Implementar e revisar: Adicionar markers de enrichment.
- [ ] Implementar e revisar: Criar capability dare-design.
- [ ] Implementar e revisar: Adicionar snapshots.

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

- [ ] Mesmo input gera mesma estrutura.
- [ ] Capability existe nos quatro harnesses.
- [ ] Conteudo fora de markers e preservado.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Design deterministico**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`024-fundacao-de-enrichment-por-ia.md`](024-fundacao-de-enrichment-por-ia.md).
