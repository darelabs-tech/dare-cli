# 036 — Reverse

**Objetivo:** Portar engenharia reversa brownfield e seus artefatos.

**Posicao na sequencia:** 36 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **reverse**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 018, 024 e 035 concluidos

## Escopo incluido

- [ ] Analisar modulos
- [ ] Implementar --deep e --modules
- [ ] Gerar IDEIA.md e specs
- [ ] Gerar report
- [ ] Usar AST opcional
- [ ] Gerar Excalidraw opcional
- [ ] Adicionar --check
- [ ] Aplicar enrichment
- [ ] Criar capability dare-reverse

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-project/src/reverse.rs`
- `crates/dare-cli/src/commands/reverse.rs`
- `assets/capabilities/dare-reverse`

## Comandos ou superficies afetadas

- `dare reverse`

## Contratos de disco afetados

- `DARE/IDEIA.md`
- `DARE/** specs`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Analisar modulos.
- [ ] Implementar e revisar: Implementar --deep e --modules.
- [ ] Implementar e revisar: Gerar IDEIA.md e specs.
- [ ] Implementar e revisar: Gerar report.
- [ ] Implementar e revisar: Usar AST opcional.
- [ ] Implementar e revisar: Gerar Excalidraw opcional.
- [ ] Implementar e revisar: Adicionar --check.
- [ ] Implementar e revisar: Aplicar enrichment.
- [ ] Implementar e revisar: Criar capability dare-reverse.

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

- [ ] Check nao escreve.
- [ ] AST e regex produzem merge estavel.
- [ ] Artefatos passam snapshots.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Reverse**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`037-dna.md`](037-dna.md).
