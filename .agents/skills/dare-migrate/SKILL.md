---
name: dare-migrate
description: Camada semântica da migração (Fase 2 brownfield). Roda depois do comando `dare migrate` e escreve a estratégia de migração + cenários Gherkin de paridade reais no DARE/MIGRATION/, garantindo reimplementação fiel ao legado.
---

# DARE Migrate Skill — Migração com paridade (brownfield Fase 2)

> **Equivalente no terminal:** `dare migrate --ai`


Você é o agente que transforma o entendimento do legado em um **plano de migração com paridade**.
Esta skill é a camada **semântica**: roda **depois** do comando `dare migrate`, que já leu os
artefatos do `reverse`/`dna` e gerou os esqueletos. Sua função é **escrever a estratégia de migração
e os cenários Gherkin de paridade reais** — o contrato comportamental que garante uma reimplementação
fiel ao legado.

> Pré-requisito: o comando `dare migrate --to <stack>` precisa ter rodado antes (gera
> `DARE/MIGRATION/MIGRATION.md`, `migration-facts.json`, `parity/*.feature`). Que por sua vez exige
> `dare reverse` já executado. Se faltar, peça ao usuário para rodar na ordem.

## Quando usar esta skill
- Projeto legado entendido (`reverse` + `dna`) que será reimplementado em outra stack.
- Acabou de rodar `dare migrate` e há seções `<!-- AGENT -->` / `# AGENT` em aberto.

## Passo a passo

### 1. Carregar contexto (não re-varrer)
Leia `migration-facts.json` (origem/alvo/módulos/blocking gaps), `IDEIA.md` + `REVERSE/module-*.md`
e `PROJECT-DNA.md`. Abra arquivos-chave do legado só o necessário para inferir comportamento.

### 2. Preencher `DARE/MIGRATION/MIGRATION.md`
- **Decisão de Paradigma** — mudou (procedural→OO, monólito→serviços)? Decisão + justificativa.
- **Estratégia** — big-bang vs. strangler/parallel-run; ordem dos módulos; feature flags.
- **Registro de Risco** — tratar cada blocking gap (🔴) + riscos de regressão/dados/performance.
- **Arquitetura-alvo** — na stack-alvo, alinhada ao DNA quando o paradigma for preservado.
- **Cutover & Rollback** — passos, validação de paridade (rodar `.feature`), go/no-go, rollback.

### 3. Gherkin de paridade (`parity/<módulo>.feature`)
Um `Scenario` por fluxo observável, derivado do **comportamento legado real**: `Given` → `When` →
`Then` idêntico ao legado. Inclua bordas/formatos (arredondamento, máscaras). É o contrato de aceite
da reimplementação na stack-alvo.

### 4. Apresentar ao usuário
Resumo: paradigma, estratégia, nº de cenários de paridade, blocking gaps a resolver.

## Regras de ouro
1. **Paridade primeiro** — todo fluxo crítico vira `Scenario`.
2. **Blocking gaps são bloqueantes** — 🔴 não resolvido é risco; trate ou registre.
3. **Respeite o DNA** — arquitetura-alvo segue as convenções da casa.
4. **Não invente comportamento** — cenário sem base no legado = regressão silenciosa.
5. **Strangler quando possível** — incremental com parallel-run reduz risco.

## Antipatterns
| AP | Antipattern | Por quê |
|---|---|---|
| AP-01 | Gherkin genérico sem base no legado | Não garante paridade |
| AP-02 | Ignorar blocking gaps (🔴) | Reimplementa sobre incerteza |
| AP-03 | Big-bang sem necessidade | Maximiza risco |
| AP-04 | Arquitetura-alvo desalinhada do DNA | Código vira ilha inconsistente |
| AP-05 | Reescrever os fatos determinísticos do CLI | Quebra a fonte de verdade |

---

Skill MIT — parte do DARE Method. Fase 2 (brownfield). Pareia com o comando `dare migrate`.
