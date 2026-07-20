---
id: ADR-001
title: "Fixture ADR válido para verify-adr-frontmatter"
status: Accepted
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance", "fixture"]
---

## Contexto

Fixture mínimo Accepted usado pelos testes de governança do microplano 001.

## Decisão

Manter frontmatter e headings na ordem exigida por DARE/BLUEPRINT.md §4.2.

## Consequências

O verificador deve aceitar este ficheiro sem erros.

## Critérios de aceite

- `verifyAdrFile` retorna lista vazia de erros para este fixture.

## Referências

- DARE/BLUEPRINT.md §5.2
- DARE/DESIGN.md
