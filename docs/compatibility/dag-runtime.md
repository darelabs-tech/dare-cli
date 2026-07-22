# DAG runtime — ranks, state store e canvas (Ciclo 026)

Runtime **library-first** no crate `dare-dag`: ranks longest-path, cascading skip, state store com `FileLock`, e canvas Markdown. Complementa [DEC-027](../DECISION-LOG.md) e o Blueprint 026. Sem CLI `dare execute` / `dare dag viz` neste ciclo.

Source: `crates/dare-dag/src/{graph,status,state,canvas}.rs`

## Ranks (0-based longest-path)

| Regra | Semântica |
|-------|-----------|
| Root | `depends_on` vazio → **rank 0** |
| Dependente | `rank(t) = 1 + max(rank(deps))` |
| Ciclo | `DagGraphError::Cycle { path }` — sem inventar ranks |
| Missing dep | `DagGraphError::MissingDependency` |

`compute_ranks` **não** corre validate completo (pré-condição: DAG preferencialmente já validado em 020). Use `compute_ranks_validated` quando quiser `validate_dag` + ranks.

## Cascading skip

Após persistência (`ensure_state` / `transition`) e via API pura `apply_cascading_skip`:

- Só tasks com status **`PENDING`** podem virar `SKIPPED` por cascade.
- **`RUNNING` nunca** é auto-skipped.
- `DONE` / `FAILED` / `SKIPPED` permanecem intocados pelo fixpoint.
- Fixpoint determinístico (ordem lexico de ids por round); segunda passagem sem mudanças → idempotente.

## Paths e lock

| Artefacto | Path relativo ao project root |
|-----------|-------------------------------|
| Runtime state | `.dare/state.json` (`STATE_REL`) |
| Canvas | `DARE/.canvas.md` (`CANVAS_REL`) |
| Lock | `.dare/state.json.darelock` (`FileLock::try_acquire` sobre `STATE_REL`) |

Contenção: **fail-fast** — segundo `try_acquire` → `CoreError::io("file lock held")` (sem fila).

## Clock

| Tipo | Uso |
|------|-----|
| `Clock` | Trait: `now_rfc3339()` para `updated_at` / timestamps |
| `SystemClock` | Wall-clock UTC via `SystemTime` |
| `FixedClock(String)` | Determinístico (testes / goldens) |

## API pública (`dare_dag`)

| Função / tipo | Papel |
|---------------|-------|
| `compute_ranks` | Longest-path ranks → `BTreeMap<id, u32>` |
| `compute_ranks_validated` | `validate_dag` (errors → `InvalidDag`) + ranks |
| `tasks_by_rank` | Agrupa ids por rank (ids lexico dentro do bucket) |
| `next_executable` | `PENDING` com todas deps `DONE`; ordem rank↑, id lexico |
| `ensure_state` | Lock → merge tasks do DAG → cascade → save |
| `transition` | Lock → mutação (`Start`/`Complete`/`Fail`/`Reset`/`Skip`) → cascade → save; opcional `RefreshCanvas` |
| `apply_cascading_skip` | Cascade puro in-memory; retorna nº de mudanças |
| `canvas::render` / `canvas::write` | Markdown do canvas; write atómico em `CANVAS_REL` |
| `TaskStatus` / `Transition` / `RefreshCanvas` | Wire status + mutações |

Status wire (case-sensitive): `PENDING` \| `RUNNING` \| `DONE` \| `FAILED` \| `SKIPPED`.

## Dependency pin: `proptest`

Workspace pin: **`proptest = "=1.6.0"`** (dev-dep de `dare-dag`).

O Blueprint 026 mencionava `=1.36.0`; essa versão **não existe** no crates.io. A implementação corrige para **1.6.0** (DEC-027).

## Fora de escopo (026)

| Item | Microplano |
|------|------------|
| `dare dag viz` (Mermaid / CLI) | **027** |
| `dare execute` (status/next/watch/complete/fail/…) | **028+** |

Validate (`dare validate`) permanece no ciclo **020** / DEC-021.

## Local verify

```bash
cargo test -p dare-dag
cargo clippy -p dare-dag -- -D warnings
```

## Container (mp026-001)

```bash
docker compose -f docker-compose.ci.yml config
```

Compose CI reutilizado (`Dockerfile.rust` + `docker-compose.ci.yml` do 003) — **verificado**, sem waiver.
