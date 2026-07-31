# 007 — Contratos persistidos

**Objetivo:** Modelar e testar todos os formatos de disco que formam o contrato do DARE.

**Posicao na sequencia:** 7 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **contratos persistidos**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 002, 004 e 005 concluidos

## Escopo incluido

- [ ] Implementar DareConfig com serde flatten
- [ ] Implementar DagV21 e LegacyDag
- [ ] Implementar RuntimeStateV1
- [ ] Implementar GraphNode e GraphEdge
- [ ] Implementar SkillsManifest
- [ ] Implementar VerificationBaseline
- [ ] Implementar UpdateManifestV1
- [ ] Implementar TelemetrySnapshot
- [ ] Criar readers/writers canonicos

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-contracts`

## Contratos de disco afetados

- `dare.config.json`
- `dare-graph.yml`
- `DARE/dare-dag.yaml`
- `.dare/state.json`
- `.dare/skills.yml`
- `.dare/verification/*.json`

Qualquer alteracao de schema, nome, ID canonico ou exit code deve ser coberta por teste de compatibilidade e, quando breaking, por ADR e migration note.

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar DareConfig com serde flatten.
- [ ] Implementar e revisar: Implementar DagV21 e LegacyDag.
- [ ] Implementar e revisar: Implementar RuntimeStateV1.
- [ ] Implementar e revisar: Implementar GraphNode e GraphEdge.
- [ ] Implementar e revisar: Implementar SkillsManifest.
- [ ] Implementar e revisar: Implementar VerificationBaseline.
- [ ] Implementar e revisar: Implementar UpdateManifestV1.
- [ ] Implementar e revisar: Implementar TelemetrySnapshot.
- [ ] Implementar e revisar: Criar readers/writers canonicos.

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

- [ ] Fixtures legadas desserializam.
- [ ] Round-trip preserva campos desconhecidos.
- [ ] Ordenacao e formatacao sao deterministicas.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Contratos persistidos**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`008-configuracao-e-migrations.md`](008-configuracao-e-migrations.md).
