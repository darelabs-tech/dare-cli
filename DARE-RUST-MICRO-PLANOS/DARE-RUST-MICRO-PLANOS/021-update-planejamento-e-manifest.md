# 021 — Update: planejamento e manifest

**Objetivo:** Criar o mecanismo de comparacao entre assets instalados e assets da nova versao.

**Posicao na sequencia:** 21 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **update: planejamento e manifest**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 008 a 014 concluidos

## Escopo incluido

- [ ] Ler UPDATE-MANIFEST schema 1
- [ ] Definir manifest novo versionado
- [ ] Classificar identical/missing/apply/customized
- [ ] Calcular SHA-256
- [ ] Criar UpdatePlan
- [ ] Implementar --dry-run e --target
- [ ] Cobrir Codex explicitamente

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-update/src/plan.rs`

## Comandos ou superficies afetadas

- `dare update --dry-run`
- `dare update --target <harness>`

## Contratos de disco afetados

- `templates/UPDATE-MANIFEST.json`
- `.dare/**`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Ler UPDATE-MANIFEST schema 1.
- [ ] Implementar e revisar: Definir manifest novo versionado.
- [ ] Implementar e revisar: Classificar identical/missing/apply/customized.
- [ ] Implementar e revisar: Calcular SHA-256.
- [ ] Implementar e revisar: Criar UpdatePlan.
- [ ] Implementar e revisar: Implementar --dry-run e --target.
- [ ] Implementar e revisar: Cobrir Codex explicitamente.

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

- [ ] Dry-run descreve exatamente as mudancas.
- [ ] Customizacoes sao detectadas.
- [ ] Codex participa do plano.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Update: planejamento e manifest**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`022-update-aplicacao-backup-e-migrations.md`](022-update-aplicacao-backup-e-migrations.md).
