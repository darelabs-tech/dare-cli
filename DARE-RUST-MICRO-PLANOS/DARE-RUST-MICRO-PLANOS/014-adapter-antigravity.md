# 014 — Adapter Antigravity

**Objetivo:** Implementar .antigravityrules e Agent Skills.

**Posicao na sequencia:** 14 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **adapter antigravity**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 005, 009 e 010 concluidos

## Escopo incluido

- [ ] Detectar .antigravityrules
- [ ] Gerar regras dinamicas
- [ ] Instalar 48 Agent Skills ou matriz revisada
- [ ] Criar .agents/workflows quando necessario
- [ ] Validar frontmatter name/description
- [ ] Tratar compartilhamento de .agents/skills com Codex

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-harness/src/antigravity.rs`

## Contratos de disco afetados

- `.antigravityrules`
- `.agents/skills/**`
- `.agents/workflows/**`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Detectar .antigravityrules.
- [ ] Implementar e revisar: Gerar regras dinamicas.
- [ ] Implementar e revisar: Instalar 48 Agent Skills ou matriz revisada.
- [ ] Implementar e revisar: Criar .agents/workflows quando necessario.
- [ ] Implementar e revisar: Validar frontmatter name/description.
- [ ] Implementar e revisar: Tratar compartilhamento de .agents/skills com Codex.

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

- [ ] Todas as skills declaradas sao materializadas.
- [ ] Frontmatter valido.
- [ ] Instalacao simultanea com Codex funciona.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Adapter Antigravity**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`015-pipeline-de-release-nativo-alpha.md`](015-pipeline-de-release-nativo-alpha.md).
