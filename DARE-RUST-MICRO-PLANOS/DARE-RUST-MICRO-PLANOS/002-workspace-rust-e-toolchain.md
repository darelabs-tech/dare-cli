# 002 — Workspace Rust e toolchain

**Objetivo:** Criar o workspace nativo que sustentara toda a reescrita.

**Posicao na sequencia:** 2 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **workspace rust e toolchain**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 001 concluido

## Escopo incluido

- [ ] Criar Cargo workspace
- [ ] Fixar rust-toolchain.toml
- [ ] Criar crates iniciais dare-cli, dare-core, dare-contracts, dare-config e dare-assets
- [ ] Configurar rustfmt, clippy e deny warnings na CI
- [ ] Definir MSRV
- [ ] Adicionar licenca, CODEOWNERS e convencoes de commits
- [ ] Criar binario dare com --help e --version

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `Cargo.toml`
- `rust-toolchain.toml`
- `crates/dare-cli`
- `crates/dare-core`
- `crates/dare-contracts`
- `crates/dare-config`
- `crates/dare-assets`

## Comandos ou superficies afetadas

- `dare --help`
- `dare --version`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Criar Cargo workspace.
- [ ] Implementar e revisar: Fixar rust-toolchain.toml.
- [ ] Implementar e revisar: Criar crates iniciais dare-cli, dare-core, dare-contracts, dare-config e dare-assets.
- [ ] Implementar e revisar: Configurar rustfmt, clippy e deny warnings na CI.
- [ ] Implementar e revisar: Definir MSRV.
- [ ] Implementar e revisar: Adicionar licenca, CODEOWNERS e convencoes de commits.
- [ ] Implementar e revisar: Criar binario dare com --help e --version.

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

- [ ] Workspace compila em modo debug e release.
- [ ] CLI responde help/version.
- [ ] Dependencias entre crates seguem a regra arquitetural.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Workspace Rust e toolchain**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`003-ci-cross-platform-e-qualidade.md`](003-ci-cross-platform-e-qualidade.md).
