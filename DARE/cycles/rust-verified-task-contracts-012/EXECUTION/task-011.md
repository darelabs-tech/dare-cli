# Task 011: Suites negativas, segurança, round-trip npm e documentação

> **Complexity:** HIGH · **depends_on:** `task-010`, `task-001` · **rank:** 6
> **Blueprint:** [§8, §9](../BLUEPRINT.md) · **Aceite:** casos 7, 8, 9, 10, 11 · **Objetivo:** O-04, O-05

## Objetivo

Fechar o ciclo: as suites transversais que nenhuma task isolada cobre, e a documentação que torna o contrato público.

## Arquivos

| Arquivo | Ação |
|---|---|
| `tests/security/handoff_security.rs` | **criar** |
| `tests/golden/handoff_compat.rs` | **criar** |
| `docs/compatibility/handoff-contracts.md` | **criar** |
| `docs/compatibility/classification-matrix.md` | editar — linha Classe B do exit 6 |
| `docs/DECISION-LOG.md` | editar — DEC-059 |
| `DARE/cycles/rust-verified-task-contracts-012/PROOF.md` | **criar** |
| `DARE/cycles/rust-verified-task-contracts-012/REGRESSION.md` | **criar** |

## Especificação

### `tests/security/handoff_security.rs`

| Teste | Cenário |
|---|---|
| `refuses_absolute_artifact_ref` | `artifact_refs: ["/etc/passwd"]` → `HANDOFF_REF_ESCAPE` |
| `refuses_dotdot_traversal` | `artifact_refs: ["../../outside.txt"]` → `HANDOFF_REF_ESCAPE` |
| `refuses_windows_drive_path` | `artifact_refs: ["C:\\Windows\\x"]` → `HANDOFF_REF_ESCAPE` |
| `symlink_not_followed` | `#[cfg(unix)]` — symlink dentro de `allowedPaths` apontando para fora: contado em `skipped_symlinks`, handoff ainda verifica |
| `secrets_never_reach_sidecar` | `AKIA...`, `token=abc`, `Authorization: Bearer x` em `deliverable`, `nextSteps`, `openItems` e `contract.owner` — nenhum aparece no arquivo em disco |
| `oversize_contract_refused` | Contrato de 3 MiB → cap de 2 MiB |
| `oversize_sidecar_refused` | Sidecar de 3 MiB → cap de 2 MiB |
| `tree_digest_ref_cap_refused` | Árvore acima de `MAX_ARTIFACT_REFS` |

### `tests/golden/handoff_compat.rs`

| Teste | O que prova |
|---|---|
| `strict_off_byte_identical` | Replay dos fixtures da `task-001` com todo o código de handoff presente e `handoff.strict` ausente: stdout e exit code **idênticos byte a byte** aos snapshots commitados. **É o teste de O-04.** |
| `strict_refusal_exits_6` | Exit 6, `error.kind == "GuardFail"` |
| `compat_exits_0` | Exit 0 com aviso em stderr |
| `npm_roundtrip_preserves_sidecar` | Ver abaixo |
| `npm_executor_ignores_receipts` | Com o `state.json` reescrito e `strict` off, composição como hoje; com `strict` on, o lado Rust recusa contexto não verificável |

**Round-trip npm — como simular sem o npm instalado.** Reescrever `.dare/state.json` mantendo **apenas** os campos que `packages/cli/src/dag-runner/state-store.ts::saveState` preserva (`version`, `updatedAt`, `tasks{status,output,error,attempts,…}`) e descartando chaves desconhecidas. Então assertar que (a) o sidecar está intocado e ainda verifica, e (b) o `state.json` reescrito continua carregando por `load_runtime_state`. Isso prova RF-10 sem exigir Node no CI.

**Os 27 goldens de `init`** ([`init_npm_parity_tests.rs`](../../../../crates/dare-cli/src/commands/init_npm_parity_tests.rs)) são reexecutados **sem edição**. Precisam continuar verdes; editá-los para fazer passar é reprovação.

### `docs/compatibility/handoff-contracts.md`

Contrato público do ciclo, no estilo de [`dag-runtime.md`](../../../../docs/compatibility/dag-runtime.md):

- Schemas em disco do contrato e do sidecar, em tabelas de campo
- Os **13** códigos de recusa, com significado de cada um
- Chaves de política e seus defaults
- Tabela de exit code para `strict` on/off
- A estratégia de artefatos A-06 (tri-state de revisão)
- **Declaração explícita:** o CLI npm compõe contexto **sem** verificação, e isso é comportamento de compatibilidade esperado, não bug

### `docs/DECISION-LOG.md` — DEC-059

Um parágrafo denso, no estilo exato de DEC-058, nomeando: crates tocados; a divisão de storage (`DARE/<ciclo>/contracts/` versionado vs. `.dare/handoff/` sidecar); `strict` opt-in com default `false`; os **dois** digests de contrato; o tri-state de revisão; os 13 códigos; `GuardFail` exit 6 como Classe B; **zero dependência nova**; link para `compatibility/handoff-contracts.md`. E o que ficou **fora**: RF-06, RF-07, RF-08, RF-11 → PR 02.

### `PROOF.md` e `REGRESSION.md`

`PROOF.md`: saída **real** de cada gate do Ralph Loop. `REGRESSION.md`: as suites pré-existentes reexecutadas e seus resultados.

> **Registre números observados, nunca números presumidos.** Se a suite tem 968 testes antes e 1.040 depois, escreva o que o comando imprimiu. Afirmar uma contagem não observada invalida o PROOF inteiro — é exatamente o erro que a IDEIA §0 aponta ao distinguir "resultados documentados" de "testes repetidos nesta análise".

## Validation Gates

Ralph Loop completo, no workspace inteiro:

```bash
cargo fmt --all                                          # antes de commitar
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
cargo deny check
```

- [ ] Todos os 7 gates verdes, com saída copiada para `PROOF.md`
- [ ] 27 goldens de `init` verdes e **não editados** (`git diff` vazio nesse arquivo)
- [ ] `cargo audit` e `cargo deny` idênticos ao baseline — A-08 prevê zero dependência nova, divergência aqui indica erro de manifesto
- [ ] `dare validate` no DAG do ciclo: 0 erros, 0 warnings

## Testes

Além das duas suites novas, reexecutar e registrar: suite completa do workspace, 27 goldens de `init`, suites de `tests/{golden,security,cross-platform}` do harness `dare-parity`.

## Segurança

Esta task **é** o gate de segurança do ciclo. Cobre OWASP A01 (confinamento), A02 (digest ≠ assinatura, afirmado na doc), A03 (segredo fora do sidecar), A08 (adulteração) e A09 (recusa auditável). `cargo audit` + `cargo deny check` cobrem A06.

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhum `assert!(true)` nem teste sem assertion
- [ ] Nenhum teste `#[ignore]` sem justificativa escrita no próprio teste
- [ ] Nenhum golden de `init` editado
- [ ] `PROOF.md` contém saída **real** colada, não paráfrase nem número presumido
- [ ] Toda contagem em `PROOF.md`/`REGRESSION.md` foi observada na execução deste ciclo
- [ ] `handoff-contracts.md` documenta os 13 códigos — nenhum faltando
- [ ] DEC-059 declara o que ficou fora de escopo
- [ ] `dare review task-011` sem findings

## Próxima Task

Nenhuma. Fim do ciclo: PR em **Draft**, sem merge sem autorização humana específica.
