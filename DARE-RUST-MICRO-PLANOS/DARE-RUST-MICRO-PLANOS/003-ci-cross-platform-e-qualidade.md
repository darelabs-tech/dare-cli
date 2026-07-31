# 003 — CI cross-platform e qualidade

**Objetivo:** Automatizar build, testes, lint e artefatos em Linux, macOS e Windows.

**Posicao na sequencia:** 3 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **ci cross-platform e qualidade**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 002 concluido

## Escopo incluido

- [ ] Criar workflow de pull request
- [ ] Executar cargo fmt --check, clippy e test
- [ ] Adicionar targets Linux x64/ARM64, macOS Intel/ARM64 e Windows x64
- [ ] Adicionar cache de Cargo
- [ ] Gerar artefatos temporarios de CI
- [ ] Adicionar cargo audit e cargo deny
- [ ] Criar smoke test do binario

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `.github/workflows/ci.yml`
- `.github/workflows/build.yml`
- `deny.toml`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Criar workflow de pull request.
- [ ] Implementar e revisar: Executar cargo fmt --check, clippy e test.
- [ ] Implementar e revisar: Adicionar targets Linux x64/ARM64, macOS Intel/ARM64 e Windows x64.
- [ ] Implementar e revisar: Adicionar cache de Cargo.
- [ ] Implementar e revisar: Gerar artefatos temporarios de CI.
- [ ] Implementar e revisar: Adicionar cargo audit e cargo deny.
- [ ] Implementar e revisar: Criar smoke test do binario.

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

- [ ] PR falha em formatacao, lint, teste ou vulnerabilidade critica.
- [ ] Todos os targets minimos compilam.
- [ ] Smoke test executa o binario produzido.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **CI cross-platform e qualidade**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`004-erros-tracing-e-saida-da-cli.md`](004-erros-tracing-e-saida-da-cli.md).
