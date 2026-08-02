# O Método DARE

**Design. Architect. Review. Execute.**

O Método DARE é uma metodologia para desenvolvimento de software assistido por IA que resolve um problema fundamental: como manter a velocidade da IA sem perder controle e auditabilidade?

---

## As 4 Fases

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│   1. DESIGN     →  2. ARCHITECT  →  3. REVIEW   →  4. EXECUTE           │
│   ─────────        ─────────────    ─────────      ──────────            │
│   Humano           IA propõe        Humano          IA implementa         │
│   define           arquitetura      valida          + Ralph Loop          │
│   requisitos                        e aprova                              │
│                                                                          │
│   ↓ DESIGN.md      ↓ BLUEPRINT.md   ↓ ✓ approval    ↓ Code + Tests ✓     │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

| Fase | O que faz | Quem faz | Saída |
|------|-----------|----------|-------|
| **1. Design** | Define **o que** construir e **por quê** | Humano (IA auxilia) | `DARE/DESIGN.md` |
| **2. Architect** | Decide **como** construir: arquitetura, endpoints, tasks | IA propõe, humano valida | `DARE/BLUEPRINT.md` |
| **3. Review** | Aprova ou ajusta o plano antes de gastar tokens | Humano | ✓ approval explícito |
| **4. Execute** | Implementa task por task com Ralph Loop até gates passarem | IA | Código + testes verdes |

> 💡 **Princípio central:** humanos pensam estratégia (fases 1 e 3), IA executa tática (fases 2 e 4). Cada transição entre fases requer checkpoint explícito — não há avanço automático.

---

## Por que o DARE funciona?

### Problema: Vibe Coding vs Tradicional

| Vibe Coding | Tradicional |
|---|---|
| "Me dá um código que faça X" + esperança | Especificação detalhada feita só por humanos |
| Rápido para protótipo, **caos para evoluir** | Lento, **aproveita pouco a IA** |
| Sem auditabilidade do raciocínio | Sem ganho de produtividade real |

### Solução DARE

O DARE preenche o gap:

1. **Contexto persistido**: cada fase gera artefatos (`DESIGN.md`, `BLUEPRINT.md`) que alimentam a fase seguinte — a IA nunca começa do zero
2. **Checkpoints humanos**: as fases 1 e 3 exigem aprovação explícita — sem "surpresas" após horas de execução
3. **Execução determinística**: o DAG de tasks garante paralelismo seguro e estado rastreável
4. **Ralph Loop**: a IA itera automaticamente até os gates (testes, linter) passarem

---

## O Ralph Loop

O **Ralph Loop** é o ciclo de auto-correção da fase 4 (Execute):

```
  ┌──────────┐
  │ Implementa│
  │ a task   │
  └────┬─────┘
       │
       ▼
  ┌──────────┐     Gates OK?    ┌──────────┐
  │  Roda os ├────────Yes──────►│  Done ✓  │
  │  Gates   │                  └──────────┘
  └────┬─────┘
       │ No
       ▼
  ┌──────────┐
  │  Lê o   │
  │  erro   │
  └────┬─────┘
       │
       └──────────────────────► (volta ao topo)
```

**Gates de validação** executados automaticamente:
- `cargo test` / testes unitários da stack
- `cargo clippy` / linter
- `cargo fmt --check` / formatter
- Type checker da stack

---

## Artefatos gerados

| Artefato | Fase | Propósito |
|---|---|---|
| `DARE/DESIGN.md` | 1 | Requisitos funcionais e não-funcionais |
| `DARE/BLUEPRINT.md` | 2 | Arquitetura, endpoints, modelo de dados |
| `DARE/TASKS.md` | 2 | Lista de tasks atômicas |
| `DARE/dare-dag.yaml` | 2 | Grafo de dependências entre tasks |
| `DARE/EXECUTION/task-*.md` | 4 | Spec individual de cada task |
| `DARE/TELEMETRY.md` | 4 | Tokens consumidos, tempo, tentativas |

---

## Leitura adicional

- [Fase 1 — Design](design-phase.md)
- [Fase 2 — Architect](architect-phase.md)
- [Fase 3 — Review](review-phase.md)
- [Fase 4 — Execute](execute-phase.md)
- [Ralph Loop](ralph-loop.md)
