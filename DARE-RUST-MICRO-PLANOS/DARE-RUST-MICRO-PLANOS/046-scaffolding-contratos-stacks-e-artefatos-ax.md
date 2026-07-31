# 046 — Scaffolding: contratos, stacks e artefatos AX

**Objetivo:** Construir a infraestrutura comum antes dos comandos init e bootstrap.

**Posicao na sequencia:** 46 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **scaffolding: contratos, stacks e artefatos ax**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 007 a 10 e 22 concluidos

## Escopo incluido

- [ ] Definir StackScaffolder trait
- [ ] Registrar 11 stack IDs
- [ ] Modelar backend/frontend/MCP/toolchain/transport
- [ ] Portar templates de stacks
- [ ] Gerar sete artefatos AX
- [ ] Criar plan/apply/rollback
- [ ] Validar outputs por stack
- [ ] Adicionar fixtures greenfield

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-scaffold`
- `assets/stacks/**`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir StackScaffolder trait.
- [ ] Implementar e revisar: Registrar 11 stack IDs.
- [ ] Implementar e revisar: Modelar backend/frontend/MCP/toolchain/transport.
- [ ] Implementar e revisar: Portar templates de stacks.
- [ ] Implementar e revisar: Gerar sete artefatos AX.
- [ ] Implementar e revisar: Criar plan/apply/rollback.
- [ ] Implementar e revisar: Validar outputs por stack.
- [ ] Implementar e revisar: Adicionar fixtures greenfield.

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

- [ ] Todas as 11 stacks possuem metadata.
- [ ] Sete artefatos AX sao testados.
- [ ] Scaffold parcial faz rollback.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Scaffolding: contratos, stacks e artefatos AX**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`047-init-e-bootstrap.md`](047-init-e-bootstrap.md).
