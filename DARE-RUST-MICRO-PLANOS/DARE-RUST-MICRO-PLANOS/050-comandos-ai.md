# 050 — Comandos ai

**Objetivo:** Expor diagnostico e execucao dos providers de enrichment.

**Posicao na sequencia:** 50 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **comandos ai**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 024 e drivers necessarios concluidos

## Escopo incluido

- [ ] Implementar ai doctor
- [ ] Implementar ai providers
- [ ] Implementar ai run
- [ ] Implementar ai prompt
- [ ] Exibir capabilities por provider
- [ ] Adicionar JSON
- [ ] Aplicar timeouts e redaction
- [ ] Cobrir provider ausente e malformed output

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-cli/src/commands/ai.rs`

## Comandos ou superficies afetadas

- `dare ai doctor`
- `dare ai providers`
- `dare ai run`
- `dare ai prompt`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar ai doctor.
- [ ] Implementar e revisar: Implementar ai providers.
- [ ] Implementar e revisar: Implementar ai run.
- [ ] Implementar e revisar: Implementar ai prompt.
- [ ] Implementar e revisar: Exibir capabilities por provider.
- [ ] Implementar e revisar: Adicionar JSON.
- [ ] Implementar e revisar: Aplicar timeouts e redaction.
- [ ] Implementar e revisar: Cobrir provider ausente e malformed output.

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

- [ ] Doctor diferencia ausente/invalido/pronto.
- [ ] Prompt nao vaza env.
- [ ] Mock permite CI deterministica.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Comandos ai**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`051-dashboard-e-rest-compativel.md`](051-dashboard-e-rest-compativel.md).
