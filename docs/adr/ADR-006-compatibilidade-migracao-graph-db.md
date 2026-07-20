---
id: ADR-006
title: "Compatibilidade migração graph DB"
status: Accepted
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance", "graphrag", "compatibility"]
---

## Contexto

O GraphRAG do DARE persiste o grafo de conhecimento em disco dentro do projeto. A baseline TypeScript 3.18.1 usa dois backends selecionáveis via `dare-graph.yml`: SQLite em `.dare/graph.db` e JSON em `.dare/graph.json`. Embeddings semânticos são armazenados como BLOB binário no SQLite; a ordem de bytes e o tipo escalar fazem parte do contrato de compatibilidade binária entre a implementação legada e o port Rust.

Projetos brownfield já possuem `.dare/graph.db` ou `.dare/graph.json` populados. Qualquer alteração de schema, encoding de vetores ou comportamento de upgrade que reescreva ou converta arquivos sem aviso explícito corrompe índices, invalida fixtures de paridade e quebra integrações que dependem de IDs canônicos estáveis.

## Decisão

1. **Paths canônicos.** O armazenamento local do grafo permanece exclusivamente em `.dare/graph.db` (SQLite, backend default) e `.dare/graph.json` (backend JSON). Nenhum outro path substitui ou redireciona silenciosamente esses arquivos; mudanças de localização exigem ADR e migration note.

2. **BLOB de vetores.** Enquanto a compatibilidade binária com a baseline 3.18.1 for exigida, colunas de embedding no SQLite permanecem BLOB de `f32` em ordem **little-endian** (LE). Leitores Rust (`rusqlite`) e legados (`sql.js`) devem interpretar o mesmo layout byte a byte; troca para outro dtype ou endianness é breaking change.

3. **Proibição de migração silenciosa.** O CLI **não** converte, reescreve nem “atualiza” `.dare/graph.db` ou `.dare/graph.json` automaticamente na abertura. Toda evolução de schema exige migration explícita (comando ou script versionado), entrada registrada no changelog e classificação na matriz de compatibilidade (CI-003). Upgrades forward-only; rollback manual via backup antes de executar migrate.

4. **Leitura legada obrigatória.** Enquanto o suporte à baseline estiver ativo, implementações MUST abrir e ler cópias de stores legados (SQLite e JSON) sem exigir reindexação prévia. Mutations sobre cópias de `.dare/graph.db` legado devem preservar schema, BLOB f32 LE e IDs canônicos de nós/arestas. Detecção de formato legado ocorre na leitura; falha de import reporta versão detectada e ação recomendada (migrate explícito), nunca conversão implícita.

## Consequências

- Positivas: paridade verificável com fixtures golden da baseline; rusqlite abre `.dare/graph.db` existente sem rewrite total; contrato de disco alinhado ao microplano 040 e à matriz CI-003.
- Negativas: evoluções de schema exigem trabalho de migration e documentação; mantenedores não podem “consertar” formatos antigos no hot path de leitura.
- Operacionais: `dare graph` (e equivalentes) MUST expor versão/schema do store em `--json`; shadow tests MUST validar abertura de DB legado antes de release candidate.

## Critérios de aceite

- ADR publicado com `status: Accepted` e decisão completa (sem placeholders).
- Documentação e código referenciam `.dare/graph.db` e `.dare/graph.json` como únicos paths locais do grafo.
- Testes de contrato confirmam leitura de BLOB `f32` LE idêntica à baseline em amostras de fixture.
- Nenhum caminho de código reescreve graph store na abertura; migrations possuem versão, changelog e nota de breaking quando aplicável.
- Suite de compatibilidade abre cópia de `.dare/graph.db` legado, executa leitura e mutação, e valida IDs e schema inalterados em relação à golden TypeScript.

## Referências

- `DARE/BLUEPRINT.md` §5.5 (ADR-006 — Graph DB)
- `DARE/DESIGN.md` RF-05
- `docs/compatibility/disk-and-json-policy.md` (CI-003)
- `docs/compatibility/breaking-change-process.md`
- Microplanos 040–043 (GraphRAG storage, ingest e semântica)
- `DARE-RUST-MICRO-PLANOS/040-graphrag-storage-e-compatibilidade.md`
