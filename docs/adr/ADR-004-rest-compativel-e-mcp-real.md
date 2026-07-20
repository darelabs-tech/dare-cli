---
id: ADR-004
title: "REST compatível e MCP real"
status: Accepted
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance", "rest", "mcp", "transport"]
---

## Contexto

O ecossistema DARE expõe contexto e telemetria por dois caminhos distintos na baseline 3.18.1:

1. **REST legado** — o pacote `dare-mcp-server` é um servidor **Express HTTP** com endpoints REST read-only (por exemplo, `POST /context/query`). Não implementa o Model Context Protocol.
2. **MCP real** — transporte conforme a especificação MCP (JSON-RPC sobre stdio, SSE ou transportes equivalentes documentados), com contrato de tools/resources/prompts distinto do HTTP REST.

O nome histórico `dare-mcp-server` induz confusão: o binário legado **não** fala MCP. O CLI nativo Rust não deve fundir nem trocar esses transportes de forma implícita. A implementação concreta de cada caminho fica para os microplanos **051** (dashboard e REST compatível) e **052** (MCP real como transporte separado); este ADR trava apenas a semântica e as regras de coexistência.

## Decisão

1. **`dare-mcp-server` legado = Express REST.** Trata-se de API HTTP compatível com a baseline 3.18.1. Não é, nem será retratado como, servidor MCP (JSON-RPC/stdio/SSE).
2. **Transportes distintos.** REST compatível e MCP real são camadas de transporte separadas, versionadas e testadas de forma independente. Compartilham lógica de negócio apenas via bibliotecas internas explícitas — nunca por substituição de protocolo.
3. **Proibição de substituição silenciosa.** É **proibido** redirecionar chamadas REST para MCP (ou o inverso) sem opt-in explícito do operador, flag documentada, changelog e, quando aplicável, ADR de breaking change. Trocar o protocolo por trás do mesmo comando, URL ou nome de binário sem aviso conta como breaking change (ver BLUEPRINT §5.6, item 5).
4. **Alias ou wrapper somente com janela de transição.** Um binário alias (por exemplo, apontar `dare-mcp-server` para o novo transporte MCP) só é permitido se houver **janela de transição documentada**: datas de início/fim, mensagem de depreciação visível, guia de migração e testes de paridade entre REST legado e MCP durante o período. Fora da janela, os dois transportes permanecem identificáveis e instaláveis separadamente.
5. **Escopo deste ADR.** Detalhes de endpoints REST, dashboard e servidor MCP entram nos ciclos **051** e **052**; aqui fixamos que REST ≠ MCP e que nenhum ciclo posterior pode violar essa distinção.

## Consequências

- O REST compatível permanece disponível enquanto a baseline 3.18.1 exigir paridade; contratos HTTP (status codes, paths, payloads) seguem a política de compatibilidade Classe A.
- O MCP real será entregue como artefato ou modo de execução **separado** do binário principal do CLI, sem reutilizar o nome `dare-mcp-server` para denotar MCP sem a janela de transição acima.
- Shadow tests e fixtures devem cobrir REST e MCP de forma isolada; falha de paridade em um transporte não autoriza desativar o outro silenciosamente.
- Documentação, `--help` e mensagens de erro devem nomear o transporte correto (`REST`, `HTTP`, `MCP`, `stdio`, etc.) para evitar que integrações assumam JSON-RPC onde só existe HTTP.

## Critérios de aceite

- [ ] ADR-004 com `status: Accepted` e distinção explícita REST (Express) vs MCP (JSON-RPC/stdio/SSE).
- [ ] Nenhum comando ou release substitui REST por MCP (ou inverso) sem flag opt-in, changelog e regra de breaking change.
- [ ] Microplanos **051** (REST compatível) e **052** (MCP real) referenciados como ciclos de implementação; semântica deste ADR não reimplementada neles de forma contraditória.
- [ ] Qualquer alias/wrapper de binário inclui janela de transição documentada (depreciação, migração, paridade).
- [ ] Endpoints REST compatíveis preservam status codes e payloads da baseline 3.18.1 até ADR de breaking change dedicado.

## Referências

- `DARE/BLUEPRINT.md` §5.5 (ADR-004), §5.6 item 5 (substituição silenciosa REST↔MCP)
- `DARE/DESIGN.md` RF-04
- Microplanos **051** — dashboard e REST compatível; **052** — MCP real como transporte separado
- `docs/compatibility/baseline-3.18.1.md` — contratos HTTP legados
- `docs/compatibility/breaking-change-process.md` — processo quando REST ou MCP mudarem de contrato
