# Fase 1 — Design

A fase de Design é onde **o humano define o que vai ser construído e por quê**. A IA pode auxiliar, mas a responsabilidade de aprovação é humana.

## Objetivo

Produzir o `DARE/DESIGN.md` — um documento de requisitos que serve como contrato entre humanos e IA para todo o ciclo de desenvolvimento.

## Comando

```bash
dare design "descrição do que você quer construir"

# Modo interativo (requer TTY)
dare design --interactive
```

## O que o Design captura?

| Seção | Conteúdo |
|---|---|
| Descrição | O problema que está sendo resolvido |
| Objetivos e métricas | Critérios mensuráveis de sucesso |
| Stakeholders | Quem usa e quem aprova |
| Requisitos Funcionais | O que o sistema deve fazer (MUST / SHOULD / COULD) |
| Requisitos Não-Funcionais | Performance, segurança, manutenibilidade |
| Restrições | Stack, idioma, compatibilidade |

## Estrutura do DESIGN.md

```markdown
# DESIGN: [Nome do projeto]

## 1. DESCRIÇÃO
...

<!-- AGENT:BEGIN section="description" -->
...
<!-- AGENT:END section="description" -->

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO
...

## 3. STAKEHOLDERS
...

## 4. REQUISITOS FUNCIONAIS
| ID | Requisito | Prioridade | Critério de aceite |
...

## 5. REQUISITOS NÃO-FUNCIONAIS
...
```

Os marcadores `<!-- AGENT:BEGIN -->` e `<!-- AGENT:END -->` delimitam seções que podem ser enriquecidas por IA (comando `dare ai`).

## Seções enriquecíveis por IA

- `description` — Refinamento da descrição
- `objectives` — Sugestão de métricas
- `functional-requirements` — Expansão de requisitos
- `stack` — Recomendação de stack técnica

## Exit codes

| Código | Quando |
|---|---|
| `0` | Sucesso |
| `1` | Erro interno |
| `2` | Uso incorreto (`--interactive` sem TTY) |
| `4` | Input inválido (descrição vazia, oversize >32KB) |
| `5` | Erro de I/O |

## Próximo passo

Após revisar o `DARE/DESIGN.md`, prossiga para a [Fase 2 — Architect](architect-phase.md):

```bash
dare blueprint
```
