# 006 — Execucao segura de processos

**Objetivo:** Substituir child_process por um executor Rust seguro e cancelavel.

**Posicao na sequencia:** 6 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **execucao segura de processos**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 002 e 004 concluidos

## Escopo incluido

- [ ] Criar SafeCommand com argv separado
- [ ] Aplicar environment allowlist
- [ ] Remover SECRET, TOKEN, KEY e PASSWORD
- [ ] Capturar stdout/stderr com limite
- [ ] Implementar timeout e exit code 124
- [ ] Implementar cancelamento e kill de arvore de processos
- [ ] Normalizar erros de executavel ausente
- [ ] Adicionar mock process runner

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-core/src/process.rs`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Criar SafeCommand com argv separado.
- [ ] Implementar e revisar: Aplicar environment allowlist.
- [ ] Implementar e revisar: Remover SECRET, TOKEN, KEY e PASSWORD.
- [ ] Implementar e revisar: Capturar stdout/stderr com limite.
- [ ] Implementar e revisar: Implementar timeout e exit code 124.
- [ ] Implementar e revisar: Implementar cancelamento e kill de arvore de processos.
- [ ] Implementar e revisar: Normalizar erros de executavel ausente.
- [ ] Implementar e revisar: Adicionar mock process runner.

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

- [ ] Nenhum comando usa shell concatenado.
- [ ] Timeout retorna 124.
- [ ] Processos filhos nao ficam orfaos.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Execucao segura de processos**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`007-contratos-persistidos.md`](007-contratos-persistidos.md).
