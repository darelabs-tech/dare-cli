---
id: ADR-001
title: "Compatibilidade de bugs legados"
status: Accepted
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance"]
---

## Contexto

A migração do DARE CLI TypeScript 3.18.1 para a implementação Rust exige paridade controlada com a baseline documentada em `docs/compatibility/baseline-3.18.1.md`. Nem todo desvio da baseline deve ser preservado: alguns são contratos públicos (Classe A), outros bugs cosméticos (B), comportamentos potencialmente explorados por integrações (C) ou vulnerabilidades (D).

Sem classificação explícita, correções legítimas seriam bloqueadas por “paridade com bug” ou, inversamente, mudanças de contrato passariam sem revisão. Este ADR formaliza as classes A/B/C/D, as ações `preserve`, `fix`, `adr_required` e `must_fix`, e os invariantes de segurança RS-06/RS-07 exigidos pelo Design Apêndice A e pelo Doc Mestre §44.

## Decisão

### Classes de compatibilidade

| Classe | Nome | Ação | Descrição |
|--------|------|------|-----------|
| **A** | Contrato público | `preserve` | Exit codes, nomes de comandos/flags, schemas persistidos, IDs canônicos e demais superfícies públicas estáveis. Alteração **somente** via `docs/compatibility/breaking-change-process.md` (BLUEPRINT §5.6): ADR Accepted, fixture de regressão, shadow test, semver major quando aplicável e aprovação humana. |
| **B** | Bug cosmético | `fix` | Desvios sem valor de contrato (texto incorreto, mojibake, formatação). Corrigir na implementação Rust e documentar na matriz; **não** exige ADR dedicada. |
| **C** | Bug comportamental potencialmente utilizado | `adr_required` | Comportamento legado que scripts ou usuários podem depender. Exige ADR Accepted (ou referência cruzada), migration note e, quando couber, fixture antes de alterar. |
| **D** | Vulnerabilidade | `must_fix` | Falhas de segurança. Correção **obrigatória**; **nunca** preservadas por paridade com a baseline, mesmo que o legado TypeScript as reproduza. |

### Regras invariantes

1. **Classe D nunca por paridade** — Itens classificados como Classe D **nunca** são preservados em nome da compatibilidade com bugs legados. A política de paridade não se aplica a vulnerabilidades (RS-07).
2. **Classe A só via breaking change** — Mudanças em itens Classe A seguem exclusivamente o processo em `docs/compatibility/breaking-change-process.md`; PRs que alterem Classe A sem ADR Accepted e checklist completo são bloqueados.
3. **Matriz canônica** — Todo item conhecido de incompatibilidade é registrado em `docs/compatibility/classification-matrix.md` com `item_id`, `class`, `action` e `adr_ref` quando aplicável.

### Exemplos referenciados (CI-005 … CI-014)

| item_id | class | action | Resumo |
|---------|-------|--------|--------|
| CI-005 | B | `fix` | Texto `dare new` incorreto no welcome |
| CI-006 | B | `fix` | Mojibake / formatação inconsistente |
| CI-007 | C | `adr_required` | Skill update/remove incompletos → este ADR |
| CI-008 | C | `adr_required` | Diferenças de JSON / ordenação → ADR-002 |
| CI-009 | C | `adr_required` | Idioma misto PT/EN → language-policy |
| CI-010 | D | `must_fix` | Path escape / symlink abuse |
| CI-011 | D | `must_fix` | Shell concatenado / execução insegura |
| CI-012 | D | `must_fix` | Secret leakage em logs/erros |
| CI-013 | D | `must_fix` | Extração insegura de arquivo (zip-slip) |
| CI-014 | D | `must_fix` | Assinatura ausente/inválida em releases/skills |

Itens CI-001 … CI-004 (Classe A, `preserve`) complementam a matriz e obedecem ao breaking-change-process quando alterados.

### Invariantes de segurança (RS-06, RS-07)

Independentemente da classe do item na matriz, a implementação Rust mantém:

- **Path safety (RS-06)** — Resolução de caminhos dentro de roots permitidos; rejeição de path escape, symlink abuse e zip-slip (CI-010, CI-013).
- **Argv separado (RS-06)** — Invocação de processos com argumentos em vetor; proibição de shell concatenado ou interpolação insegura (CI-011).
- **Redação de secrets (RS-06)** — Tokens, chaves e credenciais nunca aparecem em logs, stderr ou payloads de erro expostos ao usuário; mensagens genéricas substituem valores sensíveis (CI-012).

Classe D agrupa vulnerabilidades cuja correção é mandatória (RS-07); waiver de paridade aplica-se **apenas** a Classe D, nunca a A/B/C usadas para justificar regressão de segurança.

## Consequências

- PRs de migração devem citar `item_id`(s) da matriz e a classe correspondente; correções Classe D não aguardam shadow test de paridade com comportamento inseguro.
- Correções Classe B podem entrar sem bump major; Classe C exige ADR + migration note antes do merge.
- Alterações Classe A exigem ADR, fixtures e processo de breaking change; regressões acidentais em CI-001 … CI-004 são tratadas como falha de compatibilidade.
- CI-007 permanece vinculado a este ADR; demais itens C/D referenciam ADR-002, language-policy ou políticas de segurança conforme a matriz.
- Harness de verificação (`verify-adr-frontmatter`, matriz CI-001 … CI-014) valida presença de classificação e status Accepted deste documento.

## Critérios de aceite

- [ ] `status: Accepted` no frontmatter deste arquivo.
- [ ] `docs/compatibility/classification-matrix.md` lista CI-001 … CI-014 sem linha “não classificado”; CI-007 referencia ADR-001.
- [ ] Classe D documentada com regra explícita de não preservação por paridade; Classe A amarrada a `breaking-change-process.md`.
- [ ] Invariantes path safety, argv separado e redação de secrets citados na § Decisão (RS-06, RS-07).
- [ ] Shadow tests cobrem casos Classe A/C legados antes de fixes comportamentais; Classe D coberta por testes de segurança, não por paridade com bug.
- [ ] Nenhum exemplo contém valores reais de secrets — apenas nomes de variáveis de ambiente ou classes de vulnerabilidade.

## Referências

- `DARE/DESIGN.md` — Apêndice A (mapa das classes); RS-06, RS-07
- `DARE/BLUEPRINT.md` — §4.3 (`CompatibilityClassItem`), §5.5 (conteúdo obrigatório ADR-001), §5.6 (breaking change)
- `docs/compatibility/classification-matrix.md` — matriz CI-001 … CI-014
- `docs/compatibility/breaking-change-process.md` — fluxo para Classe A
- `docs/compatibility/baseline-3.18.1.md` — baseline de referência TypeScript 3.18.1
- Doc Mestre §44 — política de bugs legados e Classe D
