# 047 — Init e bootstrap

**Objetivo:** Entregar criacao greenfield e execucao do scaffolder configurado.

**Posicao na sequencia:** 47 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **init e bootstrap**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 011 a 15, 22 e 46 concluidos

## Escopo incluido

- [ ] Implementar init interativo
- [ ] Implementar --non-interactive
- [ ] Suportar --stack, --fullstack, --mcp, --transport, --toolchain
- [ ] Instalar harnesses
- [ ] Implementar bootstrap --force
- [ ] Garantir idempotencia
- [ ] Adicionar rollback
- [ ] Criar golden trees por stack

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-cli/src/commands/init.rs`
- `crates/dare-cli/src/commands/bootstrap.rs`

## Comandos ou superficies afetadas

- `dare init [nome]`
- `dare bootstrap`

## Contratos de disco afetados

- `dare.config.json`
- `projeto gerado`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar init interativo.
- [ ] Implementar e revisar: Implementar --non-interactive.
- [ ] Implementar e revisar: Suportar --stack, --fullstack, --mcp, --transport, --toolchain.
- [ ] Implementar e revisar: Instalar harnesses.
- [ ] Implementar e revisar: Implementar bootstrap --force.
- [ ] Implementar e revisar: Garantir idempotencia.
- [ ] Implementar e revisar: Adicionar rollback.
- [ ] Implementar e revisar: Criar golden trees por stack.

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

- [ ] Cada stack gera arvore valida.
- [ ] Non-interactive e reproduzivel.
- [ ] Bootstrap repetido nao corrompe projeto.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Init e bootstrap**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`048-hooks-e-steering.md`](048-hooks-e-steering.md).
