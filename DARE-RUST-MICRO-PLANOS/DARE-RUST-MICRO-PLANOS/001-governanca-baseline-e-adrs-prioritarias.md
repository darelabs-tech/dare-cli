# 001 — Governanca, baseline e ADRs prioritarias

**Objetivo:** Preparar as decisoes arquiteturais e a baseline observavel antes de escrever dominio complexo.

**Posicao na sequencia:** 1 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **governanca, baseline e adrs prioritarias**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Documento mestre aprovado
- [ ] Acesso ao CLI TypeScript 3.18.1 e ao repositorio

## Escopo incluido

- [ ] Registrar a versao TypeScript de referencia e seu hash
- [ ] Criar ADR-001, ADR-002, ADR-004, ADR-006 e ADR-007
- [ ] Classificar contratos publicos, bugs cosmeticos, bugs comportamentais e vulnerabilidades
- [ ] Definir politica de idioma, JSON, versionamento de disco e compatibilidade
- [ ] Criar registro de decisoes e responsaveis
- [ ] Definir processo de aprovacao de breaking changes

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `docs/adr`
- `docs/compatibility`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Registrar a versao TypeScript de referencia e seu hash.
- [ ] Implementar e revisar: Criar ADR-001, ADR-002, ADR-004, ADR-006 e ADR-007.
- [ ] Implementar e revisar: Classificar contratos publicos, bugs cosmeticos, bugs comportamentais e vulnerabilidades.
- [ ] Implementar e revisar: Definir politica de idioma, JSON, versionamento de disco e compatibilidade.
- [ ] Implementar e revisar: Criar registro de decisoes e responsaveis.
- [ ] Implementar e revisar: Definir processo de aprovacao de breaking changes.

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

- [ ] Todos os ADRs prioritarios aprovados.
- [ ] Nenhuma ambiguidade sobre contratos de compatibilidade.
- [ ] Baseline 3.18.1 identificada de forma reproduzivel.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Governanca, baseline e ADRs prioritarias**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`002-workspace-rust-e-toolchain.md`](002-workspace-rust-e-toolchain.md).
