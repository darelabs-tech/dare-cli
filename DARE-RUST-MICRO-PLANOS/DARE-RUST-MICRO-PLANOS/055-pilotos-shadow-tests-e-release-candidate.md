# 055 — Pilotos, shadow tests e release candidate

**Objetivo:** Validar a versao Rust em projetos reais antes do cutover.

**Posicao na sequencia:** 55 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **pilotos, shadow tests e release candidate**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 054 concluido

## Escopo incluido

- [ ] Selecionar projetos piloto
- [ ] Executar Rust em paralelo sem mutar quando aplicavel
- [ ] Comparar outputs e operacao diaria
- [ ] Coletar incidentes e gaps
- [ ] Congelar features TypeScript exceto seguranca
- [ ] Publicar RC
- [ ] Bloquear mudancas de contrato
- [ ] Validar rollback operacional

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `docs/pilot`
- `docs/release-candidate`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Selecionar projetos piloto.
- [ ] Implementar e revisar: Executar Rust em paralelo sem mutar quando aplicavel.
- [ ] Implementar e revisar: Comparar outputs e operacao diaria.
- [ ] Implementar e revisar: Coletar incidentes e gaps.
- [ ] Implementar e revisar: Congelar features TypeScript exceto seguranca.
- [ ] Implementar e revisar: Publicar RC.
- [ ] Implementar e revisar: Bloquear mudancas de contrato.
- [ ] Implementar e revisar: Validar rollback operacional.

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

- [ ] Projetos piloto concluem fluxos principais.
- [ ] Nenhum bloqueador P0/P1.
- [ ] Rollback testado por operadores.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Pilotos, shadow tests e release candidate**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`056-cutover-stable-e-encerramento-do-legado.md`](056-cutover-stable-e-encerramento-do-legado.md).
