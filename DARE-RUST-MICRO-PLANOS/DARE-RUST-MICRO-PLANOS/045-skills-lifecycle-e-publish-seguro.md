# 045 — Skills lifecycle e publish seguro

**Objetivo:** Implementar add, remove, update e publish corrigindo inconsistencias legadas.

**Posicao na sequencia:** 45 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **skills lifecycle e publish seguro**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 044 concluido

## Escopo incluido

- [ ] Instalacao atomica em packages/skills
- [ ] Atualizar manifest e conteudo
- [ ] Remover arquivos com protecao de reverse dependencies
- [ ] Criar bundles tar seguros
- [ ] Validar MIT e dare_version
- [ ] Publicar artefato, hash e assinatura
- [ ] Bloquear zip/tar traversal
- [ ] Documentar incompatibilidades intencionais

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-skills/src/install.rs`
- `crates/dare-skills/src/publish.rs`
- `crates/dare-cli/src/commands/skill.rs`

## Comandos ou superficies afetadas

- `dare skill add`
- `dare skill remove`
- `dare skill update`
- `dare skill publish`

## Contratos de disco afetados

- `packages/skills/**`
- `.dare/skills.yml`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Instalacao atomica em packages/skills.
- [ ] Implementar e revisar: Atualizar manifest e conteudo.
- [ ] Implementar e revisar: Remover arquivos com protecao de reverse dependencies.
- [ ] Implementar e revisar: Criar bundles tar seguros.
- [ ] Implementar e revisar: Validar MIT e dare_version.
- [ ] Implementar e revisar: Publicar artefato, hash e assinatura.
- [ ] Implementar e revisar: Bloquear zip/tar traversal.
- [ ] Implementar e revisar: Documentar incompatibilidades intencionais.

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

- [ ] Remove apaga arquivos corretos.
- [ ] Update recopia conteudo.
- [ ] Publish envia artefato verificavel.
- [ ] Extração maliciosa e bloqueada.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Skills lifecycle e publish seguro**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`046-scaffolding-contratos-stacks-e-artefatos-ax.md`](046-scaffolding-contratos-stacks-e-artefatos-ax.md).
