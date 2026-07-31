# 054 — Hardening de paridade e seguranca

**Objetivo:** Fechar diferencas TypeScript x Rust e executar testes de seguranca completos.

**Posicao na sequencia:** 54 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **hardening de paridade e seguranca**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Todos os comandos planejados implementados

## Escopo incluido

- [ ] Executar golden suite completa
- [ ] Comparar exit/stdout/stderr/tree/content/DB/state/HTTP
- [ ] Revisar normalizacoes permitidas
- [ ] Executar fuzzing de parsers e paths
- [ ] Testar command injection e env leak
- [ ] Testar archive traversal e signature mismatch
- [ ] Medir startup, memoria e tamanho
- [ ] Resolver ou documentar cada diferenca

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `tests/golden`
- `tests/security`
- `tests/cross-platform`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Executar golden suite completa.
- [ ] Implementar e revisar: Comparar exit/stdout/stderr/tree/content/DB/state/HTTP.
- [ ] Implementar e revisar: Revisar normalizacoes permitidas.
- [ ] Implementar e revisar: Executar fuzzing de parsers e paths.
- [ ] Implementar e revisar: Testar command injection e env leak.
- [ ] Implementar e revisar: Testar archive traversal e signature mismatch.
- [ ] Implementar e revisar: Medir startup, memoria e tamanho.
- [ ] Implementar e revisar: Resolver ou documentar cada diferenca.

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

- [ ] Zero diferenca nao aprovada.
- [ ] Sem vulnerabilidade critica aberta.
- [ ] Metas de performance registradas.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Hardening de paridade e seguranca**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`055-pilotos-shadow-tests-e-release-candidate.md`](055-pilotos-shadow-tests-e-release-candidate.md).
