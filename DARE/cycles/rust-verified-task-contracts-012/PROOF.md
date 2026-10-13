# PROOF — ciclo `rust-verified-task-contracts-012`

Saída real dos 7 gates do Ralph Loop, executados no workspace inteiro em
**2026-10-13**, no commit que fecha a `task-010` (`8001e5c`) com as duas suites
novas da `task-011` já presentes.

**Todo número neste documento foi observado na execução deste ciclo.** Nada
aqui é presumido, parafraseado ou herdado de um run anterior. Os totais de
teste vêm de **uma única** invocação de `cargo test --workspace
--no-fail-fast`, não de somas de execuções separadas.

Plataforma: Windows 11 Pro 10.0.26200, `x86_64-pc-windows-msvc`.

Dois tokens foram mascarados nas saídas coladas abaixo, e **somente** dois: o
caminho absoluto do projeto virou `<PROJECT_ROOT>` e o home do usuário virou
`<HOME>`. O repositório é público e nenhum dos dois é propriedade do
comportamento sob prova. Todo o resto é saída literal.

---

## Gate 1 — `cargo fmt --all -- --check`

```
exit=0
```

Sem saída. `cargo fmt --all` foi executado antes, como a spec exige.

## Gate 2 — `cargo build --workspace`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.65s
exit=0
```

## Gate 3 — `cargo test --workspace --no-fail-fast`

```
exit=0
```

Totais agregados das **77** linhas `test result` do run:

```
passed=1160  failed=0  ignored=1  suites=77
```

O único teste `ignored` é pré-existente e traz a justificativa no próprio
atributo:

```
test worktree::tests::real_git_create_remove ... ignored, requires real git on PATH
```

### Crates centrais do ciclo, no mesmo run

| Crate | Target | Passed |
|---|---|---|
| `dare-contracts` | `unittests src/lib.rs` | **59** |
| `dare-project` | `unittests src/lib.rs` | **68** |
| `dare-dag` | `unittests src/lib.rs` | **81** |
| `dare-verify` | `unittests src/lib.rs` | **119** |
| `dare-cli` | `unittests src/main.rs` | **81** |

### Suites de integração do ciclo, no mesmo run

| Suite | Task | Passed |
|---|---|---|
| `tests/handoff_baseline.rs` | task-001 | **8** |
| `tests/handoff_policy_cli.rs` | task-009 | **7** |
| `tests/handoff_emission_cli.rs` | task-010 | **11** |
| `tests/handoff_security.rs` | task-011 | **12** |
| `tests/handoff_compat.rs` | task-011 | **6** |

`handoff_baseline.rs` são os goldens gravados na `task-001`, **antes** de
qualquer código de handoff existir. Passarem intactos aqui é a medida de O-04 no
nível de CLI; `strict_off_byte_identical`, em `handoff_compat.rs`, replaya os
mesmos snapshots com a implementação completa presente.

## Gate 4 — `cargo clippy --workspace --all-targets -- -D warnings`

```
    Checking dare-cli v4.0.0 (<PROJECT_ROOT>\crates\dare-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.65s
exit=0
```

Linhas começando com `error` ou `warning` na saída: **0**.

## Gate 5 — `cargo audit`

```
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1310 security advisories (from <HOME>\.cargo\advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (516 crate dependencies)
...
warning: 2 allowed warnings found
exit=0
```

Os **2** warnings são de manutenção, não vulnerabilidades, e ambos já estavam
na allowlist antes deste ciclo:

| Crate | ID | Warning |
|---|---|---|
| `number_prefix` | `RUSTSEC-2025-0119` | unmaintained |
| `paste` | `RUSTSEC-2024-0436` | unmaintained |

Nenhum CVE HIGH ou CRITICAL. Nenhum dos dois é dependência introduzida por este
ciclo.

## Gate 6 — `cargo deny check`

```
advisories ok, bans ok, licenses ok, sources ok
exit=0
```

## Gate 7 — `dare validate` no DAG do ciclo

```
validate: ok
dagPath: DARE/cycles/rust-verified-task-contracts-012/dare-dag.yaml
format: v2.1
taskCount: 11
errorCount: 0
warningCount: 0
strict: false
mode: validate (zero mutations)
exit=0
```

---

## A-08 — zero dependência nova

`Cargo.lock` contado por `grep -c '^name = '`:

| Momento | Crates |
|---|---|
| `HEAD` antes da `task-011` (`8001e5c`) | **516** |
| Agora | **516** |

E `cargo audit` relata **516 crate dependencies** escaneadas, o mesmo número.

`globset`, `sha2` e `proptest` já estavam pinados em
`[workspace.dependencies]`. `proptest` entrou apenas como **dev-dependency** de
`dare-contracts` e `dare-verify`, exatamente como o
[BLUEPRINT.md:736](BLUEPRINT.md) previu ao registrar que ele já era dev-dep de
`dare-dag`. Nada novo foi adicionado ao grafo de dependências de runtime.

## 27 goldens de `init` — verdes e não editados

```
$ git diff --stat crates/dare-cli/src/commands/init_npm_parity_tests.rs
(vazio)
```

Diretórios de caso em `crates/dare-cli/tests/fixtures/golden/init-npm/`:
**27**, iterados pelo teste `init_matches_npm_goldens`, que afirma
`slugs.len() >= 27` e falha nomeando cada caso divergente. O arquivo de teste
não foi tocado por este ciclo; o `git diff` acima é a prova.

---

## O que estes gates não provam

- **Digest não é assinatura.** Os gates provam detecção de alteração de bytes.
  Não há autenticação de autor em nenhum ponto deste ciclo.
- **`cargo audit` fetcha a advisory-db da rede.** Um run offline usa a cópia
  local em `~/.cargo/advisory-db`, e o número de advisories carregadas pode
  diferir do **1310** registrado aqui.
- **O round-trip npm é simulado**, não executado: `.dare/state.json` é
  reescrito preservando só os campos que o `saveState` do npm preserva. Prova
  RF-10 sem exigir Node no CI, e **não** substitui um teste contra o pacote npm
  real.
- **O teste de symlink é `#[cfg(unix)]`** e não executou nesta plataforma. A
  exclusão de symlink em `digest_tree` fica coberta apenas por inspeção de
  código no Windows.
