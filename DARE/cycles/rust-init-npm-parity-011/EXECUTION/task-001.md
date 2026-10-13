# task-001 — Golden do npm com shims de comandos

## Entrega
`scripts/parity/gen-init-golden.{mjs,sh,ps1}` + `scripts/parity/shims/`; 27 casos em `crates/dare-cli/tests/fixtures/golden/init-npm/` gerados do pacote publicado `@dewtech/dare-cli@3.18.1` em `node:22-bookworm` (`--network none`); duas execuções completas byte-idênticas (`diff -r`).

## Ralph Loop
Ver `../PROOF.md` (gates do ciclo inteiro na mesma árvore).

Status: DONE
