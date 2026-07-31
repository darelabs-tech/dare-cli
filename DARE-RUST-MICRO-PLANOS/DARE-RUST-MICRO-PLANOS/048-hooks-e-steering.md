# 048 — Hooks e steering

**Objetivo:** Portar eventos, trust gate e resolucao de instrucoes por escopo.

**Posicao na sequencia:** 48 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **hooks e steering**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 005, 006 e 019 concluidos

## Escopo incluido

- [ ] Implementar eventos fechados
- [ ] Implementar allowlist de acoes
- [ ] Adicionar trusted:false default e --trust
- [ ] Garantir idempotencia SHA-256
- [ ] Implementar list/run/validate
- [ ] Ler steering frontmatter
- [ ] Resolver scope/glob/priority
- [ ] Excluir .env* obrigatoriamente

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-hooks`
- `crates/dare-steering`
- `crates/dare-cli/src/commands/hooks.rs`
- `crates/dare-cli/src/commands/steering.rs`

## Comandos ou superficies afetadas

- `dare hooks list`
- `dare hooks run <evento>`
- `dare hooks validate`
- `dare steering list`
- `dare steering show <file>`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar eventos fechados.
- [ ] Implementar e revisar: Implementar allowlist de acoes.
- [ ] Implementar e revisar: Adicionar trusted:false default e --trust.
- [ ] Implementar e revisar: Garantir idempotencia SHA-256.
- [ ] Implementar e revisar: Implementar list/run/validate.
- [ ] Implementar e revisar: Ler steering frontmatter.
- [ ] Implementar e revisar: Resolver scope/glob/priority.
- [ ] Implementar e revisar: Excluir .env* obrigatoriamente.

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

- [ ] Evento desconhecido retorna 2.
- [ ] Hook nao confiavel nao executa.
- [ ] .env nunca entra no steering.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Hooks e steering**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`049-verificacao-avancada-e-bench.md`](049-verificacao-avancada-e-bench.md).
