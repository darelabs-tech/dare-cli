# Ralph Loop

O **Ralph Loop** é o ciclo de auto-correção automática que acontece dentro da Fase 4 (Execute). É o mecanismo central que permite à IA iterar até que todos os gates de qualidade passem.

---

## A Inspiração

Inspirado no **Ralph Wiggum** dos Simpsons — que mesmo em perigo, continua tentando de forma inocente e persistente. O Ralph Loop usa essa força: agentes de IA são ruins em planejamento estratégico, mas excelentes em iteração até o objetivo.

---

## Como funciona

```
1. IA implementa a task e escreve o código
        │
        ▼
2. CLI roda os Validation Gates automaticamente
   (testes unitários, linter, formatter, type checker)
        │
        ├── Todos passaram? ──► Task DONE ✓
        │
        └── Algum falhou?
                │
                ▼
        3. IA lê o erro completo
                │
                ▼
        4. IA corrige o código
                │
                └──────────────► (volta ao passo 2)
```

O loop continua **ininterruptamente** até:
- Todos os gates passarem (sucesso), ou
- O budget de tentativas/tokens ser esgotado (falha)

---

## Gates de Validação

Os gates são determinísticos e executados na seguinte ordem:

| Ordem | Gate | Rust | Node | Python |
|---|---|---|---|---|
| 1 | Formatter | `cargo fmt --check` | `prettier --check` | `ruff format --check` |
| 2 | Linter | `cargo clippy -D warnings` | `eslint` | `ruff check` |
| 3 | Type check | (via clippy) | `tsc --noEmit` | `mypy` |
| 4 | Testes | `cargo test` | `jest` | `pytest` |
| 5 | Audit | `cargo audit` | `npm audit` | `pip-audit` |

---

## Por que o Ralph Loop funciona?

Os agentes de IA têm duas características complementares:

| Fraqueza | Força |
|---|---|
| Ruins em planejamento estratégico de longo prazo | Excelentes em iteração local sobre erros |
| Tendem a "alucinar" sem feedback | Melhoram muito com mensagens de erro concretas |

O Ralph Loop usa a **força** dos agentes (iteração sobre erros concretos) para compensar a **fraqueza** (planejamento). O humano fez o planejamento nas fases 1 e 3; a IA executa na fase 4.

---

## Telemetria do Loop

Cada execução é rastreada em `DARE/TELEMETRY.md`:

```markdown
| Task | Modelo | Tokens (estimado) | Tempo | Tentativas |
|---|---|---|---|---|
| task-001 | claude-sonnet-4.5 | 12,450 | 3m 22s | 2 |
| task-002 | claude-sonnet-4.5 | 8,120 | 1m 48s | 1 |
```

Use `dare dashboard` para visualizar a telemetria em tempo real via browser.
