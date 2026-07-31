# 031 — Drivers reais de agentes

**Objetivo:** Integrar Codex, Claude Code, Cursor Agent e Antigravity por uma suite comum.

**Posicao na sequencia:** 31 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **drivers reais de agentes**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 006, 024 e 030 concluidos

## Escopo incluido

- [ ] Implementar Codex JSONL
- [ ] Implementar Claude Code CLI
- [ ] Implementar Cursor Agent CLI
- [ ] Implementar Antigravity CLI
- [ ] Criar doctor por driver
- [ ] Suportar command overrides
- [ ] Normalizar token/cost quando disponivel
- [ ] Redigir secrets
- [ ] Testar malformed output

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-agent/src/drivers/**`

## Comandos ou superficies afetadas

- `dare execute --agent --driver codex`
- `dare execute --agent --driver claude`
- `dare execute --agent --driver cursor`
- `dare execute --agent --driver antigravity`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar Codex JSONL.
- [ ] Implementar e revisar: Implementar Claude Code CLI.
- [ ] Implementar e revisar: Implementar Cursor Agent CLI.
- [ ] Implementar e revisar: Implementar Antigravity CLI.
- [ ] Implementar e revisar: Criar doctor por driver.
- [ ] Implementar e revisar: Suportar command overrides.
- [ ] Implementar e revisar: Normalizar token/cost quando disponivel.
- [ ] Implementar e revisar: Redigir secrets.
- [ ] Implementar e revisar: Testar malformed output.

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

- [ ] Todos passam suite detection/success/failure/timeout/cancel.
- [ ] Executavel ausente gera diagnostico.
- [ ] Nenhum secret aparece nos logs.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Drivers reais de agentes**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`032-review.md`](032-review.md).
