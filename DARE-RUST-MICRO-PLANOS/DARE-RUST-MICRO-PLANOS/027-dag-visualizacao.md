# 027 — DAG: visualizacao

**Objetivo:** Portar dare dag viz nos tres formatos suportados.

**Posicao na sequencia:** 27 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **dag: visualizacao**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 026 concluido

## Escopo incluido

- [ ] Gerar Mermaid
- [ ] Gerar DOT
- [ ] Gerar Excalidraw
- [ ] Implementar --dag, --format e --output
- [ ] Ordenar nos e edges deterministicamente
- [ ] Adicionar golden files

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-dag/src/viz.rs`
- `crates/dare-cli/src/commands/dag.rs`

## Comandos ou superficies afetadas

- `dare dag viz --format mermaid`
- `dare dag viz --format dot`
- `dare dag viz --format excalidraw`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Gerar Mermaid.
- [ ] Implementar e revisar: Gerar DOT.
- [ ] Implementar e revisar: Gerar Excalidraw.
- [ ] Implementar e revisar: Implementar --dag, --format e --output.
- [ ] Implementar e revisar: Ordenar nos e edges deterministicamente.
- [ ] Implementar e revisar: Adicionar golden files.

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

- [ ] Golden files sao estaveis.
- [ ] Formatos abrem nas ferramentas esperadas.
- [ ] Paths de output sao seguros.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **DAG: visualizacao**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`028-execute-status-next-e-watch.md`](028-execute-status-next-e-watch.md).
