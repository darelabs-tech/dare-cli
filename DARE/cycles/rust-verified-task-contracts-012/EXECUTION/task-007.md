# Task 007: `evaluate_parent` — as doze checagens ordenadas

> **Complexity:** HIGH · **depends_on:** `task-005`, `task-006` · **rank:** 3
> **Blueprint:** [§3.2 `evaluate_parent`](../BLUEPRINT.md) · **RF:** RF-05 · **Objetivo:** O-01, O-02

## Objetivo

A decisão de consumo. É o único lugar do ciclo onde "importar ou recusar" é resolvido, e é o que transforma O-01 e O-02 em comportamento.

## Dois princípios de design

**1. Não retorna `Result`.** Toda falha é um `HandoffDecision` com `RefusalCode`, porque a decisão **é** o produto. Um `Err` forçaria o chamador a distinguir "falhou ao avaliar" de "avaliou e recusou", e essas duas coisas são a mesma coisa aqui.

**2. A ordem das checagens é contrato testável.** A primeira falha ganha. Um fixture quebrado de duas formas reporta a de número menor — e existe teste para isso. Sem ordem fixa, a mensagem de recusa viraria indeterminística e o diagnóstico ficaria inútil.

## Arquivos

| Arquivo | Ação |
|---|---|
| `crates/dare-verify/src/handoff.rs` | editar — adicionar `evaluate_parent` |

## Especificação

```rust
pub fn evaluate_parent(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    parent_id: &str,
    policy: &HandoffPolicy,
) -> HandoffDecision
```

### As 12 checagens, em ordem

| # | Checagem | `RefusalCode` | `EvidenceState` |
|---|---|---|---|
| 1 | Record existe em `.dare/handoff/<id>.json` | `ReceiptMissing` | `Missing` |
| 2 | Desserializa, `schema_version == 1`, `refs.len() == digests.len()` | `ReceiptInvalid` | `Invalid` |
| 3 | Todo path do recibo é `SafeRelativePath` confinado | `RefEscape` | `Invalid` |
| 4 | Contrato existe (se `policy.require_contracts`) | `ContractMissing` | `Missing` |
| 5 | `material_digest(contrato em disco) == receipt.contract_digest` | `ContractDigestMismatch` | `Invalid` |
| 6 | `digest_file(verification_ref) == receipt.verification_digest` | `VerificationDigestMismatch` | `Invalid` |
| 7 | `gates_satisfied(report, contract.verification.required_gates)` | `GateNotSatisfied` | `Insufficient` |
| 8 | `receipt.verdict == Passed` | `VerdictNotPassed` | `Insufficient` |
| 9 | Cada `artifact_digest` bate com o arquivo **agora** | `ArtifactDigestMismatch` | `Invalid` |
| 10 | `revision.state != Unattested` (se `require_attested_revision`) | `RevisionUnattested` | `Insufficient` |
| 11 | `record.kind == policy.required_kind` | `NotFinal` | `NotApplicable` |
| 12 | Qualquer cap de tamanho atingido na leitura | `SizeLimit` | `Insufficient` |

### TOCTOU

As checagens **5, 6 e 9 releem o disco no momento da chamada**. Nenhum cache, nenhum reaproveitamento de valor lido antes no mesmo processo. Em `--agent` a composição acontece dentro do loop de tentativas ([`execute_agent.rs:259`](../../../../crates/dare-cli/src/commands/execute_agent.rs#L259)), então a revalidação ocorre **por tentativa** — é essa propriedade que faz "alterar um byte depois da verificação invalida a importação" valer na janela entre verificar e consumir.

### Sucesso

```rust
HandoffDecision {
    parent_id, state: Valid, refusal: None,
    imported_digest: Some(digest do recibo canônico),
    deliverable: Some(record.deliverable),
    detail: String::new(),
}
```

### Invariante

`state == Valid` ⟺ `refusal.is_none()` ⟺ `deliverable.is_some()`. Assertar no código e cobrir com **proptest**. Em toda falha, `deliverable` e `imported_digest` são `None`.

### `detail`

Sempre passa por `dare_core::redact` antes de retornar. Nomeia o path ofensor ou o **prefixo de 12 chars** do digest — diagnosticável sem vazar conteúdo. Um `detail` com o digest inteiro ou com o conteúdo do artefato é reprovação.

## Validation Gates

- [ ] `cargo test -p dare-verify`
- [ ] `cargo clippy -p dare-verify --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] Assinatura devolve `HandoffDecision`, **não** `Result<HandoffDecision, _>`

## Testes

13 testes mínimos — 12 negativos (um por código) mais o caminho positivo:

| Teste | Fixture |
|---|---|
| `accepts_full_valid_chain` | Tudo íntegro → `Valid` |
| `refuses_missing_record` | Sidecar ausente |
| `refuses_invalid_record` | `schema_version: 2`; e `refs`/`digests` de tamanhos diferentes |
| `refuses_ref_escape_absolute` | `artifact_refs: ["/etc/passwd"]` |
| `refuses_ref_escape_dotdot` | `artifact_refs: ["../outside.txt"]` |
| `refuses_missing_contract` | Contrato deletado, `require_contracts: true` |
| `refuses_tampered_contract` | `goal` alterado após a emissão |
| `refuses_tampered_verification` | Relatório alterado após a emissão |
| `refuses_missing_required_gate` | Relatório sem o aspecto `lint` exigido |
| `refuses_verdict_failed` | `verdict: failed` |
| `refuses_tampered_artifact` | 1 byte alterado no artefato |
| `refuses_unattested_when_required` | `state: unattested` + `require_attested_revision: true` |
| `refuses_followup_when_final_required` | `kind: follow-up-handoff` |
| `refuses_oversize_contract` | Contrato > 2 MiB |
| `check_order_lower_number_wins` | Fixture com contrato adulterado **e** artefato adulterado → reporta #5, não #9 |

**Proptest:** invariante dos 3 campos coerentes sob geração aleatória de decisões.

## Segurança

| Risco | Controle |
|---|---|
| Importar resultado adulterado (OWASP A08) | Checagens 5, 6, 9 — o núcleo do ciclo |
| TOCTOU entre verificar e consumir | Releitura no momento do consumo; por tentativa em `--agent` |
| Path escape / symlink (A01) | Checagem 3 via `SafeRelativePath` + `ProjectRoot::resolve` |
| Vazamento no diagnóstico (A09) | `detail` redigido, digest truncado em 12 chars |
| Confiar no agregado | Checagem 7 valida **por aspecto**, não `report.ok` |
| `follow-up` desbloqueando dependência final | Checagem 11 |

## Definition of Done (ANTI-STUB)

- [ ] Nenhum `TODO`, `FIXME`, `XXX`, `HACK`
- [ ] Nenhuma função vazia; nenhum `todo!()`, `unimplemented!()`
- [ ] Nenhum `unwrap()` / `expect()` em código de produção
- [ ] Nenhum `_ => HandoffDecision::default()` ou braço catch-all que engula um caso
- [ ] Todos os **13** `RefusalCode` são alcançáveis por `evaluate_parent` ou pelo preflight (`StrictNoContracts` é da `task-009`); os 12 desta task têm teste próprio
- [ ] As checagens 5, 6 e 9 leem do disco dentro desta função — nenhum parâmetro pré-computado
- [ ] Mocks apenas em `#[cfg(test)]`
- [ ] `dare review task-007` sem findings

## Próxima Task

`task-009` depende desta e de `task-008`.
