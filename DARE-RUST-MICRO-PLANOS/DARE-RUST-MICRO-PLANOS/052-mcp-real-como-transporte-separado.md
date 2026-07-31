# 052 — MCP real como transporte separado

**Objetivo:** Adicionar protocolo MCP sem quebrar a API REST legada.

**Posicao na sequencia:** 52 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **mcp real como transporte separado**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 051 concluido
- [ ] ADR-004 aprovado

## Escopo incluido

- [ ] Escolher rmcp ou implementacao equivalente
- [ ] Criar ProjectService, GraphService, DagService, TaskService e SteeringService
- [ ] Expor tools MCP
- [ ] Suportar stdio
- [ ] Suportar streamable HTTP se aprovado
- [ ] Mapear erros do dominio para MCP
- [ ] Criar testes com cliente MCP
- [ ] Manter alias dare-mcp-server durante transicao

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-server/src/mcp.rs`

## Comandos ou superficies afetadas

- `dare mcp serve --transport stdio`
- `dare mcp serve --transport streamable-http`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Escolher rmcp ou implementacao equivalente.
- [ ] Implementar e revisar: Criar ProjectService, GraphService, DagService, TaskService e SteeringService.
- [ ] Implementar e revisar: Expor tools MCP.
- [ ] Implementar e revisar: Suportar stdio.
- [ ] Implementar e revisar: Suportar streamable HTTP se aprovado.
- [ ] Implementar e revisar: Mapear erros do dominio para MCP.
- [ ] Implementar e revisar: Criar testes com cliente MCP.
- [ ] Implementar e revisar: Manter alias dare-mcp-server durante transicao.

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

- [ ] Cliente MCP descobre e executa tools.
- [ ] REST continua compativel.
- [ ] Transportes compartilham servicos de dominio.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **MCP real como transporte separado**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`053-self-update-e-package-managers.md`](053-self-update-e-package-managers.md).
