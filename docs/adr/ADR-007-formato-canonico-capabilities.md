---
id: ADR-007
title: "Formato canônico de capabilities"
status: Accepted
date: 2026-07-20
deciders: ["dare-labs"]
tags: ["governance", "capabilities", "adapters"]
---

## Contexto

O CLI DARE expõe dois conceitos distintos que hoje se confundem na documentação e no inventário de assets:

1. **Skills-pacote** — artefatos gerenciados por `dare skill` (publicação, instalação, lifecycle). Representam conhecimento reutilizável em formato de pacote (frontmatter + corpo Markdown), independente de qual IDE o desenvolvedor usa.
2. **Capabilities de IDE** — unidades de integração que cada harness de agente materializa no filesystem do projeto (slash commands, rules, skills nativos, etc.).

Os quatro harnesses suportados — **Claude**, **Cursor**, **Codex** e **Antigravity** — expõem capabilities com nomes, paths e metadados heterogêneos. Sem um modelo canônico, adapters duplicam lógica, IDs divergem entre harnesses e a paridade com a baseline npm 3.18.1 fica impossível de auditar.

Este ADR trava **apenas o contrato** do modelo canônico. A matriz YAML e os assets derivados serão criados no microplano 010 (`010-modelo-canonico-de-capabilities`).

## Decisão

### Distinção obrigatória: skill-pacote ≠ capability de IDE

| Conceito | Comando / origem | Escopo | Persistência típica |
|----------|------------------|--------|---------------------|
| Skill-pacote | `dare skill` | Conhecimento portável entre projetos e IDEs | Registro de skills DARE + pacote publicável |
| Capability de IDE | Adapter de harness | Integração nativa de um agente específico | Paths e formatos do harness (`.claude/commands/`, `.cursor/rules/`, etc.) |

Um skill-pacote **pode** ser mapeado para uma ou mais capabilities de IDE via adapter, mas nunca substitui o registro canônico: IDs, campos e validação pertencem ao modelo abaixo, não ao frontmatter bruto de cada harness.

### Campos canônicos de Capability

Toda capability registrada no DARE deve ser expressável com exatamente estes sete campos:

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `id` | string | Identificador estável, kebab-case, único no escopo do projeto; imutável após Accepted |
| `title` | string | Nome legível para humanos e help |
| `description` | string | Resumo de uma linha do propósito da capability |
| `instructions` | string (Markdown) | Corpo operacional: quando usar, passos, restrições; **sem secrets, tokens ou PII** |
| `cli_commands` | array\<string\> | Comandos CLI DARE relacionados (ex.: `dare validate`, `dare execute`) |
| `outputs` | object | Mapa harness → paths relativos gerados ou sincronizados (ex.: `claude: .claude/commands/dare-validate.md`) |
| `assets` | array\<string\> | Paths de arquivos estáticos empacotados com a capability (templates, fixtures, ícones) |

Exemplo ilustrativo (sem dados sensíveis):

```yaml
id: dare-validate
title: "Validar DAG DARE"
description: "Valida ciclos, referências e campos obrigatórios em DARE/dare-dag.yaml"
instructions: |
  Use quando o DAG mudou ou antes de commit/CI.
  Rode `dare validate` e corrija erros reportados antes de prosseguir.
cli_commands:
  - dare validate
outputs:
  claude: .claude/commands/dare-validate.md
  cursor: .cursor/commands/dare-validate.md
  codex: .codex/skills/dare-validate/SKILL.md
  antigravity: .antigravity/commands/dare-validate.md
assets: []
```

### Matriz de capabilities (escopo futuro)

A fonte de verdade agregada será `assets/capability-matrix.yml`, **criada no microplano 010** — não faz parte deste ADR. Até lá, adapters e validadores referenciam este contrato de campos sem exigir o arquivo de matriz.

### Adapters de harness

Cada adapter implementa o mapeamento capability canônica → artefatos nativos do harness:

| Harness | Responsabilidade do adapter |
|---------|------------------------------|
| **Claude** | Materializar slash commands e skills em `.claude/` |
| **Cursor** | Materializar commands, rules e skills em `.cursor/` |
| **Codex** | Materializar skills e configurações em `.codex/` |
| **Antigravity** | Materializar commands e integrações nativas em `.antigravity/` |

Adapters não inventam campos fora do schema canônico; exceções intencionais de paridade devem ser registradas na matriz (microplano 010) com justificativa explícita.

## Consequências

- Positivas: IDs estáveis permitem diff de paridade entre harnesses; CI pode validar o schema antes da matriz existir; skills-pacote e capabilities de IDE deixam de ser tratados como sinônimos.
- Negativas: adapters existentes precisarão convergir para o trait comum; breaking change em qualquer campo canônico ou em `id` exige novo ADR e bump de `schema_version`.
- Neutras: `assets/capability-matrix.yml` permanece ausente até o microplano 010; nenhum código de build deve assumir sua presença neste ciclo.

## Critérios de aceite

1. Documentação e código referenciam ADR-007 como **Accepted** para o contrato de capabilities.
2. Todo registro de capability validável contra os sete campos: `id`, `title`, `description`, `instructions`, `cli_commands`, `outputs`, `assets`.
3. Distinção skill-pacote (`dare skill`) vs capability de IDE explícita em reviews de adapter.
4. Quatro harnesses nomeados: Claude, Cursor, Codex, Antigravity.
5. Campo `instructions` livre de secrets em exemplos e fixtures de governança.
6. `assets/capability-matrix.yml` **não** criado neste ADR; criação bloqueada até microplano 010.
7. Matriz de compatibilidade CI-004 atualizada quando a matriz YAML for implementada.

## Referências

- DARE/BLUEPRINT.md §5.5 (ADR-007 — Capabilities)
- DARE-RUST-MICRO-PLANOS/010-modelo-canonico-de-capabilities.md (matriz e `assets/capability-matrix.yml`)
- docs/compatibility/classification-matrix.md (CI-004)
- docs/compatibility/baseline-3.18.1.md (inventário legado npm)
