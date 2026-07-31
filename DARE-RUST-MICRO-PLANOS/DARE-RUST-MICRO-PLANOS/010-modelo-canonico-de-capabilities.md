# 010 — Modelo canonico de capabilities

**Objetivo:** Criar a fonte unica para workflows de Claude, Cursor, Codex e Antigravity.

**Posicao na sequencia:** 10 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **modelo canonico de capabilities**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 009 concluido
- [ ] ADR-007 aprovado

## Escopo incluido

- [ ] Definir Capability e HarnessOutputs
- [ ] Criar capability-matrix.yml
- [ ] Mapear 49 Claude commands, 33 Cursor commands, 25 Cursor rules e 48 Agent Skills
- [ ] Registrar excecoes intencionais
- [ ] Validar nomes, frontmatter e duplicidade
- [ ] Gerar outputs por harness em build time

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-harness/src/capability.rs`
- `assets/capability-matrix.yml`
- `assets/capabilities/**`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Definir Capability e HarnessOutputs.
- [ ] Implementar e revisar: Criar capability-matrix.yml.
- [ ] Implementar e revisar: Mapear 49 Claude commands, 33 Cursor commands, 25 Cursor rules e 48 Agent Skills.
- [ ] Implementar e revisar: Registrar excecoes intencionais.
- [ ] Implementar e revisar: Validar nomes, frontmatter e duplicidade.
- [ ] Implementar e revisar: Gerar outputs por harness em build time.

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

- [ ] Matriz cobre 100% dos assets atuais.
- [ ] CI detecta capability ausente.
- [ ] Uma capability gera outputs reproduziveis.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Modelo canonico de capabilities**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`011-adapter-claude-code.md`](011-adapter-claude-code.md).
