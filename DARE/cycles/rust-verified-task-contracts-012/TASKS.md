# TASKS — Verified Task Contracts & Handoffs

> **Ciclo:** `rust-verified-task-contracts-012` · **PR lógico 1 de 4**
> **DESIGN:** [DESIGN.md](DESIGN.md) ✅ aprovado · **BLUEPRINT:** [BLUEPRINT.md](BLUEPRINT.md) ✅ aprovado
> **DAG:** [dare-dag.yaml](dare-dag.yaml) — `dare validate`: **0 erros, 0 warnings**
> **Decisão a registrar:** DEC-059

## Visão geral

| | |
|---|---|
| Total de tasks | **11** |
| Ranks | **7** (0–6) |
| Largura máxima | **3** tasks em paralelo (ranks 0 e 2) |
| Complexity | 1 LOW · 4 MED · 6 HIGH |
| Crates tocados | `dare-contracts`, `dare-verify`, `dare-dag`, `dare-project`, `dare-cli` |
| Dependências novas | **zero** (`sha2`, `globset` já pinados no workspace) |

## Tabela de status

| ID | Título | Status | Depends On | Complexity | Rank |
|---|---|---|---|---|---|
| task-001 | Golden baseline de `execute --next` / `--complete` | ⏳ PENDING | — | LOW | 0 |
| task-002 | `dare-contracts::handoff` — tipos, parse, validate, paths | ⏳ PENDING | — | HIGH | 0 |
| task-004 | Primitivas de proveniência Git | ⏳ PENDING | — | MED | 0 |
| task-003 | Digests determinísticos — contrato, arquivo, árvore | ⏳ PENDING | task-002 | MED | 1 |
| task-005 | Emissão do recibo e satisfação dos gates | ⏳ PENDING | task-003, task-004 | HIGH | 2 |
| task-006 | Store do sidecar — atômico, lock, supersede | ⏳ PENDING | task-003 | MED | 2 |
| task-008 | Porta `ParentEvidence` + `compose_task_prompt_verified` | ⏳ PENDING | task-003 | MED | 2 |
| task-007 | `evaluate_parent` — as doze checagens ordenadas | ⏳ PENDING | task-005, task-006 | HIGH | 3 |
| task-009 | Política, flags `--strict-handoff`, preflight estrito | ⏳ PENDING | task-007, task-008 | HIGH | 4 |
| task-010 | Emissão no `--complete`, supersede no `--reset`, JSON | ⏳ PENDING | task-009 | HIGH | 5 |
| task-011 | Suites negativas, segurança, round-trip npm, docs | ⏳ PENDING | task-010, task-001 | HIGH | 6 |

Progresso: `0/11` — ⏳ 11 PENDING · 🔄 0 RUNNING · ✅ 0 DONE · ❌ 0 FAILED

## Tasks por rank

### Rank 0 — fundação e baseline (3 em paralelo)

- **task-001** · Grava o comportamento atual como golden. **Precisa rodar antes de qualquer mudança de comportamento** — é a referência que torna O-04 verificável em vez de afirmado.
- **task-002** · Tipos, parse, validate e paths em `dare-contracts`. A fundação de A-01.
- **task-004** · `head_sha`, `worktree_is_dirty`, `changed_files` em `dare-project`. Isolada do resto.

### Rank 1 — integridade

- **task-003** · Os dois digests de contrato (A-05), `digest_file` em streaming, `digest_tree` determinístico com caps.

### Rank 2 — primitivas (3 em paralelo)

- **task-005** · Recibo e gates em `dare-verify/src/handoff.rs`.
- **task-006** · Sidecar em `dare-verify/src/handoff_store.rs` — **arquivo separado de propósito**, para que 005 e 006 nunca editem o mesmo arquivo em paralelo.
- **task-008** · Porta e composição verificada em `dare-dag`, sem dependência nova.

### Rank 3 — decisão

- **task-007** · `evaluate_parent`. 12 checagens em ordem fixa, 12 testes negativos.

### Rank 4 — política

- **task-009** · Config, flags, preflight D-06.1, wiring dos dois caminhos.

### Rank 5 — produção

- **task-010** · Emissão no complete, supersede no reset, campos JSON aditivos.

### Rank 6 — fecho

- **task-011** · Segurança, round-trip npm, documentação, DEC-059, PROOF.

## Notas de execução

**Serialização intencional de `task-009` → `task-010`.** As duas editam `commands/execute.rs`. A dependência é de propriedade de arquivo, não de output lógico, e está declarada para evitar conflito de merge em worktrees paralelos.

**`task-005` e `task-006` no mesmo rank** só é seguro porque cada uma é dona de um arquivo distinto em `dare-verify/src/`. Ambas tocam `lib.rs` com um `pub mod` de uma linha — append trivialmente mergeável.

**Cauda linear nos ranks 4–6.** Não é falso acoplamento: `task-010` precisa da política de `task-009` resolvida no mesmo arquivo, e `task-011` precisa da integração completa para comparar contra o baseline. A largura real do ciclo está nos ranks 0–2.

## Gates por task

Toda task roda o Ralph Loop do seu escopo antes de DONE:

```bash
cargo fmt --all -- --check
cargo build -p <crate>          # workspace na task-011
cargo test -p <crate>
cargo clippy -p <crate> --all-targets -- -D warnings
```

`task-011` roda o loop completo mais `cargo audit` e `cargo deny check`.

Cada spec em [`EXECUTION/`](EXECUTION/) tem **Definition of Done (ANTI-STUB)** própria. `dare review <task-id>` sem findings é condição para DONE.

## Próximas etapas

1. Revisar e aprovar este TASKS.md e o DAG.
2. `dare execute --next` para ver o rank 0 pronto, ou `/dare-execute task-001`.
3. Rank 0 em paralelo: `dare execute --parallel` (3 tasks independentes).
4. Ao fim: `PROOF.md`, `REGRESSION.md`, `DEC-059`, PR em **Draft** — sem merge sem autorização humana específica.
