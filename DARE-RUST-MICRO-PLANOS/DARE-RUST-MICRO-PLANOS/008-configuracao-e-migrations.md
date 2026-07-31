# 008 — Configuracao e migrations

**Objetivo:** Criar carregamento, validacao, defaults e migrations de configuracao.

**Posicao na sequencia:** 8 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **configuracao e migrations**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 007 concluido

## Escopo incluido

- [ ] Definir precedencia CLI/env/config/default
- [ ] Validar blocos opt-in enabled:false
- [ ] Preservar chaves desconhecidas
- [ ] Criar migration plan e dry-run
- [ ] Criar backups antes de migration
- [ ] Adicionar schema version quando autorizado
- [ ] Criar mensagens de diagnostico por path JSON

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-config`

## Contratos de disco afetados

- `dare.config.json`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir precedencia CLI/env/config/default.
- [ ] Implementar e revisar: Validar blocos opt-in enabled:false.
- [ ] Implementar e revisar: Preservar chaves desconhecidas.
- [ ] Implementar e revisar: Criar migration plan e dry-run.
- [ ] Implementar e revisar: Criar backups antes de migration.
- [ ] Implementar e revisar: Adicionar schema version quando autorizado.
- [ ] Implementar e revisar: Criar mensagens de diagnostico por path JSON.

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

- [ ] Config legada carrega sem perda.
- [ ] Migration dry-run nao escreve.
- [ ] Falhas apontam o campo exato.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Configuracao e migrations**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`009-inventario-e-empacotamento-de-assets.md`](009-inventario-e-empacotamento-de-assets.md).
