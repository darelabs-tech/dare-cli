# REGRESSION — ciclo `rust-verified-task-contracts-012`

Suites **pré-existentes** reexecutadas com toda a implementação do ciclo
presente, para responder uma pergunta: algo que funcionava antes deixou de
funcionar?

**Resposta: não.** 1160 testes passaram, 0 falharam, no mesmo run de
`cargo test --workspace --no-fail-fast` registrado em [PROOF.md](PROOF.md).

Todo número abaixo foi **observado** nessa execução, em **2026-10-13**. Nenhum
é herdado de um run anterior nem estimado.

---

## Resultado agregado

```
passed=1160  failed=0  ignored=1  suites=77
exit=0
```

O único `ignored` é pré-existente e justificado no próprio atributo:
`worktree::tests::real_git_create_remove ... ignored, requires real git on PATH`.

## Harness `dare-parity` — suites `tests/{golden,security,cross-platform}`

Nenhuma destas foi tocada pelo ciclo.

| Target | Passed |
|---|---|
| `dare-parity` `unittests src/lib.rs` | **17** |
| `tests/golden_suite.rs` | **3** |
| `tests/security_suite.rs` | **6** |
| `tests/cross_platform.rs` | **7** |
| `tests/normalize_anti_cheat.rs` | **2** |

`normalize_anti_cheat.rs` é a guarda contra normalização excessiva nos goldens.
Continuar verde importa: se este ciclo tivesse afrouxado alguma normalização
para fazer um snapshot passar, é aqui que apareceria.

## Goldens de paridade com o npm `init`

| Verificação | Resultado |
|---|---|
| Diretórios de caso em `tests/fixtures/golden/init-npm/` | **27** |
| `init_matches_npm_goldens` | verde |
| `git diff --stat` em `init_npm_parity_tests.rs` | **vazio** |

O arquivo de teste não foi editado. Editá-lo para fazer um caso passar seria
reprovação da task, e o `git diff` vazio é a prova de que não aconteceu.

## Crates alterados pelo ciclo — contagem antes e depois

As colunas "antes" vêm dos commits do próprio ciclo, cada um registrado no
`--output` da sua task no momento em que passou; a coluna "depois" vem do run
único de hoje. Onde os dois números coincidem, é porque a task que fechou aquele
crate foi a última a tocá-lo.

| Crate | Antes do ciclo | Depois | Delta | Task que alterou |
|---|---|---|---|---|
| `dare-contracts` | 28 | **59** | +31 | task-002, task-003 |
| `dare-project` | 50 | **68** | +18 | task-004 |
| `dare-dag` | 69 | **81** | +12 | task-008 |
| `dare-verify` | 65 | **119** | +54 | task-005, task-006, task-007 |
| `dare-cli` (unit) | 60 | **81** | +21 | task-009, task-010 |

> As contagens "antes" de `dare-contracts` (28), `dare-project` (50),
> `dare-dag` (69), `dare-verify` (65) e `dare-cli` (60) são **derivadas** dos
> deltas registrados nas tasks, não medidas por um checkout do commit anterior
> nesta sessão. Os números da coluna "depois" são os observados hoje. Quem
> precisar das contagens "antes" com rigor de medição deve rodar a suite no
> commit `da5f571` (ponta do `main`) e registrar o resultado.

## Suites novas do ciclo

Não são regressão — são cobertura adicionada. Registradas aqui para fechar a
soma.

| Suite | Task | Passed |
|---|---|---|
| `tests/handoff_baseline.rs` | task-001 | **8** |
| `tests/handoff_policy_cli.rs` | task-009 | **7** |
| `tests/handoff_emission_cli.rs` | task-010 | **11** |
| `tests/handoff_security.rs` | task-011 | **12** |
| `tests/handoff_compat.rs` | task-011 | **6** |

## O teste que mais importa aqui

`handoff_compat.rs::strict_off_byte_identical` replaya os quatro snapshots que a
`task-001` gravou no rank 0, **antes** de qualquer código de handoff existir,
com a implementação inteira agora presente e `handoff.strict` ausente. stdout e
exit code são comparados byte a byte.

Durante a `task-010` esse teste **pegou uma regressão real**: a primeira versão
injetava o bloco `handoff` em todo item de `ready` do `--next --json`, o que
alterou o JSON de um projeto que nem usa o recurso. A correção foi emitir o
bloco somente quando `strict` está ligado ou o projeto já produziu algum
sidecar. Sem esse golden, a mudança teria passado.

## Limites desta regressão

- Plataforma única: Windows 11, `x86_64-pc-windows-msvc`. Nada aqui diz respeito
  a Linux ou macOS.
- O teste de symlink é `#[cfg(unix)]` e **não executou**.
- O round-trip npm é simulado reescrevendo `.dare/state.json` como o
  `saveState` do npm o reescreve. Não houve execução do pacote npm real.
