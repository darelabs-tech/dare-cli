# 022 — Update: aplicacao, backup e migrations

**Objetivo:** Aplicar atualizacoes de forma segura e reversivel.

**Posicao na sequencia:** 22 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **update: aplicacao, backup e migrations**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 021 concluido

## Escopo incluido

- [ ] Implementar politicas keep/replace/ask
- [ ] Criar backup versionado
- [ ] Aplicar migrations de config
- [ ] Escrever atomicamente
- [ ] Implementar --force e -y
- [ ] Gerar report human/json
- [ ] Testar rollback em falha

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-update/src/apply.rs`
- `crates/dare-cli/src/commands/update.rs`

## Comandos ou superficies afetadas

- `dare update`
- `dare update --force`

## Contratos de disco afetados

- `.dare/backup-*`
- `dare.config.json`
- `assets instalados`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar politicas keep/replace/ask.
- [ ] Implementar e revisar: Criar backup versionado.
- [ ] Implementar e revisar: Aplicar migrations de config.
- [ ] Implementar e revisar: Escrever atomicamente.
- [ ] Implementar e revisar: Implementar --force e -y.
- [ ] Implementar e revisar: Gerar report human/json.
- [ ] Implementar e revisar: Testar rollback em falha.

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

- [ ] Nenhuma customizacao e perdida sem consentimento.
- [ ] Backup permite restauracao.
- [ ] Aplicacao parcial nao persiste.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Update: aplicacao, backup e migrations**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`023-design-deterministico.md`](023-design-deterministico.md).
