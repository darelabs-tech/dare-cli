# 011 — Adapter Claude Code

**Objetivo:** Implementar deteccao, instalacao e validacao dos artefatos Claude Code.

**Posicao na sequencia:** 11 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **adapter claude code**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 005, 009 e 010 concluidos

## Escopo incluido

- [ ] Detectar CLAUDE.md e .claude
- [ ] Gerar CLAUDE.md dinamico
- [ ] Instalar .claude/commands
- [ ] Gerar .claude/settings.json
- [ ] Configurar hook PostToolUse compativel
- [ ] Preservar customizacoes conforme politica
- [ ] Validar 49 commands ou matriz revisada

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-harness/src/claude.rs`

## Contratos de disco afetados

- `CLAUDE.md`
- `.claude/commands/**`
- `.claude/settings.json`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Detectar CLAUDE.md e .claude.
- [ ] Implementar e revisar: Gerar CLAUDE.md dinamico.
- [ ] Implementar e revisar: Instalar .claude/commands.
- [ ] Implementar e revisar: Gerar .claude/settings.json.
- [ ] Implementar e revisar: Configurar hook PostToolUse compativel.
- [ ] Implementar e revisar: Preservar customizacoes conforme politica.
- [ ] Implementar e revisar: Validar 49 commands ou matriz revisada.

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

- [ ] Instalacao idempotente.
- [ ] Arquivos correspondem a matriz.
- [ ] Projeto customizado nao e sobrescrito silenciosamente.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Adapter Claude Code**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`012-adapter-cursor.md`](012-adapter-cursor.md).
