---
id: ADR-002
title: "Contrato de saída JSON"
status: Accepted
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance", "json", "compatibility"]
---

## Contexto

Comandos DARE expõem saída estruturada via `--json` para automação, CI e integrações externas. Na baseline TypeScript 3.18.1, diferenças de ordenação de chaves, campos ausentes ou tipos inconsistentes quebram snapshots e scripts consumidores — classificado como **CI-008** (Classe C, `adr_required`).

Este ADR fecha o contrato entre writers (implementação Rust nativa), golden tests e consumidores. Ele complementa a política de disco em `docs/compatibility/disk-and-json-policy.md` e alinha-se à classificação **Classe A** (contrato público) definida em `DARE/DESIGN.md` Apêndice A.

Saídas JSON **não** devem incluir secrets, tokens, credenciais ou PII em campos documentados ou emitidos em produção; erros e telemetria seguem redação obrigatória (RS-06, RS-07).

## Decisão

### Estabilidade: `--json` como contrato Classe A

Toda chave de primeiro nível (e chaves aninhadas em objetos públicos) presente na saída `--json` de um comando documentado integra o **contrato Classe A**. Alterações nesse contrato só entram via processo de breaking change (BLUEPRINT §5.6): proposta → ADR → revisão Tech Lead/PO → changelog + migration note quando aplicável.

Comandos cobertos incluem, entre outros, saídas de `dare info`, `dare validate`, `dare graph`, harness de governança e demais subcomandos que declarem suporte a `--json` na baseline ou documentação oficial.

### Writers: ordenação lexicográfica determinística

Implementações **writers** (serialização para stdout/arquivo em modo `--json`) devem emitir JSON **canônico** para golden tests:

- Em **cada objeto JSON**, as chaves são serializadas em **ordenação lexicográfica** (comparação Unicode de strings, ordem crescente).
- Arrays mantêm ordem semântica definida pelo comando; apenas a ordenação de **keys de objetos** é normalizada.
- Fixtures e snapshots da baseline validam essa ordenação; diferença de ordem sem mudança de valor é regressão **CI-008**.

### Evolução de campos: não-breaking vs Breaking

| Mudança | Classificação |
|---------|---------------|
| Novo campo **opcional** com default seguro quando ausente (consumidor antigo ignora ou infere valor estável) | **Não-breaking** |
| Remoção de chave existente | **Breaking** |
| Renomeação de chave (mesmo semântica) | **Breaking** |
| Mudança de **tipo** de campo existente (ex.: `string` → `number`, escalar → objeto) | **Breaking** |
| Mudança de semântica de valor para mesma chave e tipo | **Breaking** (requer ADR + changelog) |

Default seguro: ausência do campo produz comportamento equivalente ao valor default documentado; consumidores que não leem o campo novo continuam corretos.

### Allowlist de campos voláteis

Campos cujo valor **pode variar entre execuções** sem indicar regressão funcional devem constar nesta allowlist. Golden tests devem ignorar ou normalizar apenas estes campos:

| Campo (caminho lógico) | Motivo |
|------------------------|--------|
| `timestamp`, `generated_at`, `*.timestamp` | Relógio de execução |
| `duration_ms`, `elapsed_ms`, `*.duration_ms` | Tempo de execução |
| `version` (quando reflete build/commit do binário em execução) | Metadado de artefato, não de contrato de domínio |
| `trace_id`, `span_id` | Correlação efêmera de tracing |

Novos campos voláteis exigem atualização desta allowlist no ADR ou em anexo referenciado; não entram silenciosamente como Classe A estável.

### Unknown keys em config de disco: preservar (flatten)

Arquivos de configuração persistidos (`dare.config.json`, `.dare/state.json` e equivalentes) podem conter chaves desconhecidas à versão atual do parser (forward compatibility, edição manual, versões futuras).

**Regra:** ao ler, parsear e regravar config de disco, o runtime **preserva** unknown keys — via merge superficial ou **flatten** de chaves extras no objeto raiz/nested conforme `docs/compatibility/disk-and-json-policy.md`. Unknown keys **não** são descartadas, renomeadas nem silenciosamente movidas para outro namespace.

Isso aplica-se a **persistência em disco**, não à saída `--json` de comandos (onde o schema é fechado e documentado).

## Consequências

- Writers Rust devem centralizar serialização JSON canônica (ordenação lexicográfica de keys) — evitar `serde_json` default sem ordenação em paths de golden test.
- CI e `scripts/governance/` passam a tratar divergência de ordenação ou schema `--json` como falha **CI-008** até ADR Accepted autorizar mudança.
- Adição de campo opcional não-breaking pode ocorrer em minor release com changelog; remoção, renomeação ou mudança de tipo exige major ou breaking process completo.
- Config migrations devem documentar tratamento de unknown keys; testes de round-trip garantem flatten/preservação.
- Documentação de API JSON não exemplifica nem incentiva logging de secrets em payloads de resposta.

## Critérios de aceite

- [ ] `status: Accepted` no frontmatter deste ADR.
- [ ] Matriz **CI-008** referencia ADR-002; classificação `adr_required` satisfeita.
- [ ] Golden tests / fixtures da baseline validam ordenação lexicográfica de keys em objetos `--json`.
- [ ] Checklist de PR para saída JSON distingue mudança não-breaking (campo opcional + default) vs Breaking (remoção, renomeação, tipo).
- [ ] Allowlist de campos voláteis aplicada nos comparadores de snapshot.
- [ ] Testes de round-trip de config com unknown keys demonstram preservação (flatten) sem perda.
- [ ] Nenhum exemplo de schema `--json` documenta campos de secret/token em texto claro.

## Referências

- `DARE/BLUEPRINT.md` §5.5 (ADR-002), §5.6 (breaking change), CI-008
- `DARE/DESIGN.md` RF-03, RF-09, RF-11; Apêndice A (Classe A)
- `docs/compatibility/disk-and-json-policy.md` — schemas persistidos e saída JSON
- `docs/compatibility/breaking-change-process.md` — fluxo de aprovação
- `docs/compatibility/classification-matrix.md` — CI-008
- `docs/compatibility/baseline-3.18.1.md` — snapshots JSON de referência
