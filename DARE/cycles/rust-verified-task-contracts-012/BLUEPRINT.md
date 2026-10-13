# BLUEPRINT — Verified Task Contracts & Handoffs

> **Ciclo:** `rust-verified-task-contracts-012` · **DESIGN:** [DESIGN.md](DESIGN.md) (aprovado em 2026-10-13)
> **Baseline:** `main` @ `da5f571` · workspace 4.0.0 · MSRV 1.88.0
> **Decisão a registrar:** DEC-059
> **Status:** aguardando aprovação humana

---

## 1. Visão geral da arquitetura

Nenhum crate novo. Nenhuma dependência externa nova. A entrega é uma **camada de prova** inserida entre três crates que já existem, com inversão de dependência no ponto de consumo.

```
                    ┌─────────────────────────────────────────┐
                    │ dare-cli :: commands/execute*.rs        │
                    │ ─ orquestra, decide política            │
                    │ ─ preflight (D-06.1)                    │
                    │ ─ impl ParentEvidence                   │
                    └───────────┬──────────────┬──────────────┘
                                │              │
              ┌─────────────────▼───┐      ┌───▼──────────────────────┐
              │ dare-dag            │      │ dare-verify              │
              │ ─ compose_task_      │      │ ─ emite receipt          │
              │   prompt_verified   │      │ ─ checa requiredGates    │
              │ ─ trait Parent      │◄─────┤ ─ evaluate_parent        │
              │   Evidence (porta)  │ impl │   (lê disco, revalida)   │
              └─────────────────┬───┘      └───┬──────────────────────┘
                                │              │
                    ┌───────────▼──────────────▼──────────────┐
                    │ dare-contracts :: handoff.rs            │
                    │ ─ tipos TaskContract / HandoffRecord    │
                    │ ─ digests (material, full, file, tree)  │
                    │ ─ paths confinados, política pura       │
                    └─────────────────┬───────────────────────┘
                                      │
                    ┌─────────────────▼───────────────────────┐
                    │ dare-core :: fs, path, redact, canonical│
                    └─────────────────────────────────────────┘
```

### Decisões arquiteturais

#### A-01 — Sem crate novo. Tipos em `dare-contracts`, avaliação em `dare-verify`

**Restrição descoberta:** `dare-dag` depende **apenas** de `dare-core` e `dare-contracts` ([`crates/dare-dag/Cargo.toml`](../../../crates/dare-dag/Cargo.toml)). Não vê `dare-verify`. E `dare-verify` depende de `dare-agent` + `globset` — é a camada de processo/verificação.

**Justificativa:** adicionar `dare-verify` a `dare-dag` não criaria ciclo (verifiquei: `dare-verify → dare-agent → dare-core`, e só `dare-cli` depende de `dare-dag`), mas inverteria o layering — o domínio do grafo passaria a depender da camada que executa processos. `dare-dag` é descrito como *library-first* em [`dag-runtime.md`](../../../docs/compatibility/dag-runtime.md) e não deve ganhar essa superfície.

**Escolha:** `dare-contracts` recebe `handoff.rs`. Ele já é o crate de contratos persistidos (`dag.rs`, `state.rs`, `config.rs`), e **os dois lados já o veem**. A avaliação que precisa ler `VerificationReport` fica em `dare-verify`, que já depende de `dare-contracts`.

| Alternativa | Veredito |
|---|---|
| Crate novo `dare-handoff` | Rejeitada. 24º membro do workspace para ~900 linhas, sem fronteira de domínio própria; os tipos pertencem ao crate de contratos persistidos |
| `dare-verify` dentro de `dare-dag` | Rejeitada. Inverte o layering do domínio do grafo (ver acima) |
| Tudo em `dare-cli` | Rejeitada. Lógica de integridade não testável como unidade de biblioteca; contraria o padrão *library-first* do repo |

#### A-02 — Inversão de dependência no consumo: `trait ParentEvidence`

`compose_task_prompt` vive em `dare-dag`, que não pode ler recibos. A porta é um trait definido em `dare-dag` e implementado em `dare-cli`:

- `dare-dag` declara `trait ParentEvidence { fn decide(&self, parent_id: &str) -> HandoffDecision; }`
- `dare-cli` implementa, delegando a `dare_verify::evaluate_parent`
- `HandoffDecision` mora em `dare-contracts` (visível aos dois)

Isso mantém `dare-dag` com a mesma árvore de dependências de hoje e deixa a composição testável com um `ParentEvidence` falso em teste unitário, sem tocar disco.

#### A-03 — `compose_task_prompt` preservada byte a byte; variante nova ao lado

A função atual **não muda de assinatura nem de comportamento**. Isso é O-04 por construção, e não por disciplina.

```rust
// inalterada — segue sendo a usada quando strict == false
pub fn compose_task_prompt(doc, state, task_id) -> Result<String, DagGraphError>

// nova
pub fn compose_task_prompt_verified(doc, state, task_id, ev) -> Result<ComposedPrompt, DagGraphError>
```

**Superfície esquecida pelo DESIGN:** `compose_task_prompt` tem **dois** chamadores, não um. Além de [`execute_agent.rs:259`](../../../crates/dare-cli/src/commands/execute_agent.rs#L259), ela é chamada por `build_next_report` em [`execution.rs:277`](../../../crates/dare-dag/src/execution.rs#L277) — ou seja, `dare execute --next` também imprime prompts compostos. Ambos os caminhos entram no escopo; o golden de `--next` com `strict = false` é o que ancora O-04.

#### A-04 — Diretório de contratos = irmão `contracts/` do DAG resolvido

O contrato precisa de um path canônico sem exigir config nova e sem o library layer conhecer o nome do ciclo. `prepare_mutation_ctx` já resolve `dag_rel` via `resolve_project_rel` ([`execute.rs:509`](../../../crates/dare-cli/src/commands/execute.rs#L509)). Então:

```
contract_dir(dag_rel) = parent(dag_rel) / "contracts"
```

| DAG | Diretório de contratos |
|---|---|
| `DARE/cycles/rust-verified-task-contracts-012/dare-dag.yaml` | `DARE/cycles/rust-verified-task-contracts-012/contracts/` |
| `DARE/dare-dag.yaml` (default) | `DARE/contracts/` |
| `--dag outro/caminho/x.yaml` | `outro/caminho/contracts/` |

Contratos viajam com o DAG, funcionam igual para raiz e ciclo, zero configuração. Resolve a parte do RF-01 que o DESIGN deixou para cá.

#### A-05 — Dois digests de contrato, não um

**Inconsistência no DESIGN corrigida aqui.** RF-02 diz que o `contractDigest` cobre o contrato e, na mesma lista, que mudança **não material** (`owner`, `requirementRefs`) só "deixa trilha auditável". Com um único digest sobre o contrato inteiro isso é contraditório: mexer em `owner` invalidaria a importação.

| Digest | Cobre | Divergência significa |
|---|---|---|
| `contractDigest` | **Projeção material**: `schemaVersion`, `taskId`, `goal`, `scope`, `acceptance`, `verification`, `exitCriteria`, `startingContext` | Mudança material → **recusa** (`HANDOFF_CONTRACT_DIGEST_MISMATCH`) |
| `contractFullDigest` | Contrato canônico inteiro, incluindo opcionais e `extra` | Mudança não material → **trilha auditável**, sem recusa |

#### A-06 — Artefatos: Git quando há, digest de árvore quando não

`scope.allowedPaths` é glob (`crates/dare-cli/src/**`). Expandir e hashear tudo seria caro e frágil: qualquer edição não relacionada dentro do glob invalidaria o handoff. Estratégia em três estados, alinhada ao tri-state de RF-09:

| Estado do Git | `artifactRefs` | `worktreeDigest` | `sourceRevision.state` |
|---|---|---|---|
| Disponível, árvore limpa | Arquivos alterados entre a revisão do `Start` e a do `Complete`, **∩ `allowedPaths`**, com digest por arquivo | ausente | `clean` |
| Disponível, árvore suja | Idem, mais modificações não commitadas dentro de `allowedPaths` | digest único da expansão ordenada dos globs | `dirty` |
| Ausente / offline | vazio | digest único da expansão ordenada dos globs | `unattested` |

Caps obrigatórios: **512** `artifactRefs` e **64 MiB** totais para `worktreeDigest`; exceder → `HANDOFF_SIZE_LIMIT`. Em estrito, `unattested` só passa se `handoff.requireAttestedRevision == false`; caso contrário `HANDOFF_REVISION_UNATTESTED`.

#### A-07 — Preflight na camada CLI, espelhando o guard

O preflight D-06.1 fica em `dare-cli`, invocado nos caminhos de `execute` antes de qualquer composição, no mesmo formato de [`dare-guard/src/preflight.rs:16`](../../../crates/dare-guard/src/preflight.rs#L16) (`run_preflight` → `CoreError::GuardFail`). Não entra em `dare-dag` porque precisa do `dag_rel` e do filesystem, que são preocupações da camada de comando.

#### A-08 — Zero dependência nova

Tudo necessário já está pinado em [`Cargo.toml`](../../../Cargo.toml) do workspace: `sha2 =0.10.9`, `serde_yaml` (`yaml_serde =0.10.4`), `globset =0.4.16`, `fs4 =1.1.0`. A única mudança de manifesto é adicionar `sha2 = { workspace = true }` a `crates/dare-contracts/Cargo.toml`. Consequência direta: `cargo audit` / `cargo deny check` não ganham superfície nova.

---

## 2. Modelo de dados em disco

### 2.1 Task Contract — `<contract_dir>/<task-id>.yaml`

Versionado no Git. YAML para combinar com `dare-dag.yaml`.

| Campo | Tipo | Obrig. | Default | Constraints |
|---|---|---|---|---|
| `schemaVersion` | `u32` | sim | — | `== 1`; outro valor → `CoreError::Config("unsupported contract schema version")` |
| `taskId` | `String` | sim | — | `task_id_is_path_safe` ✓; deve existir no DAG; deve bater com o nome do arquivo |
| `goal` | `String` | sim | — | 1..=500 chars após `trim`; não vazio |
| `scope.allowedPaths` | `Vec<String>` | sim | — | 1..=64 entradas; cada uma glob relativa válida (`globset::Glob::new`); sem `..`, sem prefixo absoluto |
| `scope.deniedPaths` | `Vec<String>` | não | `[]` | mesmas regras; avaliada depois de `allowedPaths` |
| `acceptance[]` | `Vec<AcceptanceCriterion>` | sim | — | 1..=64 itens; `id` único no contrato, regex `^[A-Z]-[0-9]{2,3}$`; `assertion` 1..=500 chars |
| `verification.requiredGates` | `Vec<GateAspect>` | sim | — | 1..=3 itens, subconjunto de `build`\|`test`\|`lint`; sem duplicatas |
| `exitCriteria.kind` | `HandoffKind` | sim | — | `verified-deliverable` \| `follow-up-handoff` |
| `startingContext.source` | `String` | sim | — | `project-revision` \| `parent-handoff` \| `none` |
| `startingContext.repositories[]` | `Vec<RepoRef>` | não | `[]` | 0..=8; `{ alias, repositoryUri? }`; `alias` regex `^[a-z][a-z0-9-]{0,31}$` |
| `taskType` | `Option<String>` | não | `None` | 1..=64 chars |
| `owner` | `Option<String>` | não | `None` | 1..=128 chars; passa por `redact` na emissão |
| `requirementRefs[]` | `Vec<String>` | não | `[]` | 0..=32; 1..=128 chars cada |
| *(desconhecidos)* | `Map<String,Value>` | — | `{}` | preservados via `#[serde(flatten)] extra`, como `DagTask` e `DareConfig` já fazem |

Limite de arquivo: `MAX_CONTRACT_BYTES` (2 MiB), herdado de `read_limited` ([`io.rs:21`](../../../crates/dare-contracts/src/io.rs#L21)).

**Exemplo concreto** — `DARE/cycles/rust-verified-task-contracts-012/contracts/task-004.yaml`:

```yaml
schemaVersion: 1
taskId: task-004
goal: "Emitir Verification Receipt derivado do VerificationReport em disco"
scope:
  allowedPaths:
    - "crates/dare-verify/src/handoff.rs"
    - "crates/dare-verify/src/lib.rs"
  deniedPaths:
    - "crates/dare-verify/src/ralph.rs"
acceptance:
  - id: A-01
    assertion: "Gate reprovado não emite recibo verified-deliverable"
  - id: A-02
    assertion: "verificationDigest bate com o arquivo .dare/verification/<id>.json lido do disco"
  - id: A-03
    assertion: "Ausência de Git produz sourceRevision.state == unattested, nunca SHA fabricado"
verification:
  requiredGates: ["build", "test", "lint"]
exitCriteria:
  kind: verified-deliverable
startingContext:
  source: project-revision
  repositories:
    - alias: dare-cli
taskType: implementation
requirementRefs: ["RF-03", "RF-09"]
```

### 2.2 Handoff + Receipt — `.dare/handoff/<task-id>.json`

Sidecar privado, JSON canônico (chaves ordenadas via `to_canonical_json_string`). Nome segue o precedente de `.dare/verification/<id>.advanced.json`.

| Campo | Tipo | Obrig. | Constraints |
|---|---|---|---|
| `schemaVersion` | `u32` | sim | `== 1` |
| `taskId` | `String` | sim | `task_id_is_path_safe` ✓ |
| `kind` | `HandoffKind` | sim | `verified-deliverable` \| `follow-up-handoff` |
| `receipt` | `VerificationReceipt` | sim | ver 2.3 |
| `deliverable` | `String` | sim | 0..=`limits.taskOutputChars`; truncado por `truncate_chars`; `redact` aplicado |
| `contextBounds` | `u32` | sim | `limits.parentContextChars` efetivo na emissão |
| `nextSteps[]` | `Vec<String>` | não | 0..=32, 1..=500 chars cada |
| `openItems[]` | `Vec<String>` | não | 0..=32; **deve** ser não vazio quando `kind == follow-up-handoff` |
| `superseded[]` | `Vec<SupersededReceipt>` | não | 0..=16 (FIFO, descarta o mais antigo); `{ receiptDigest, createdAt, reason }` |

### 2.3 `VerificationReceipt` (embutido)

| Campo | Tipo | Fonte / constraint |
|---|---|---|
| `schemaVersion` | `u32` | `1` |
| `taskId` | `String` | DAG |
| `contractDigest` | `String` | hex SHA-256 lowercase, 64 chars — projeção material (A-05) |
| `contractFullDigest` | `String` | idem, contrato inteiro |
| `sourceRevision` | `SourceRevision` | `{ state: clean\|dirty\|unattested, repositories: Vec<RepoRevision> }`; `RepoRevision { alias, commitSha: Option<String>, worktreeDigest: Option<String> }`; `commitSha` regex `^[0-9a-f]{40}$` quando presente |
| `artifactRefs[]` | `Vec<String>` | paths relativos POSIX, ordenados lexicograficamente; 0..=512 |
| `artifactDigests[]` | `Vec<String>` | paralelo a `artifactRefs`, mesmo índice; **invariante: `len()` igual** |
| `verificationRef` | `String` | `.dare/verification/<task-id>.json` |
| `verificationDigest` | `String` | SHA-256 do arquivo canônico lido do disco |
| `verdict` | `Verdict` | `passed` \| `failed` \| `insufficient` |
| `createdAt` | `String` | RFC3339; `FixedClock` nos goldens |
| `producer` | `Producer` | `{ cliVersion, driver }` — `cliVersion` = `env!("CARGO_PKG_VERSION")`; `driver` = id do driver de agente ou `"cli"` |

**Exemplo concreto** — `.dare/handoff/task-004.json` (reindentado para leitura; em disco é uma linha canônica):

```json
{
  "contextBounds": 2000,
  "deliverable": "receipt_from_verification implementado; 3 gates aprovados; 2 arquivos tocados",
  "kind": "verified-deliverable",
  "nextSteps": ["task-005 consome este recibo em evaluate_parent"],
  "openItems": [],
  "receipt": {
    "artifactDigests": [
      "9f2c1a0b7d4e8f3a5c6b2d1e0f9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a",
      "1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f809"
    ],
    "artifactRefs": [
      "crates/dare-verify/src/handoff.rs",
      "crates/dare-verify/src/lib.rs"
    ],
    "contractDigest": "4e1f8a9b0c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f",
    "contractFullDigest": "b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8",
    "createdAt": "2026-10-13T12:00:00Z",
    "producer": { "cliVersion": "4.0.0", "driver": "cli" },
    "schemaVersion": 1,
    "sourceRevision": {
      "repositories": [
        { "alias": "dare-cli", "commitSha": "da5f571ede84d168ad8d94780e7cdcc3796c0ab2", "worktreeDigest": null }
      ],
      "state": "clean"
    },
    "taskId": "task-004",
    "verdict": "passed",
    "verificationDigest": "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4",
    "verificationRef": ".dare/verification/task-004.json"
  },
  "schemaVersion": 1,
  "superseded": [],
  "taskId": "task-004"
}
```

### 2.4 Configuração — `dare.config.json`

Lida de `DareConfig.extra`, **nunca escrita pelo `init`**. Ausência total ≡ todos os defaults.

| Chave | Tipo | Default | Efeito |
|---|---|---|---|
| `handoff.strict` | `bool` | `false` | Liga *fail closed* no consumo |
| `handoff.requireAttestedRevision` | `bool` | `false` | Em estrito, `unattested` passa a ser recusa |
| `handoff.requireContracts` | `bool` | `true` quando `strict` | Em estrito, task sem contrato é recusa |

---

## 3. Superfície de API pública

### 3.1 `dare-contracts::handoff`

```rust
pub const CONTRACT_SCHEMA_VERSION: u32 = 1;
pub const HANDOFF_SCHEMA_VERSION:  u32 = 1;
pub const HANDOFF_DIR_REL:  &str = ".dare/handoff";
pub const CONTRACT_DIR_NAME: &str = "contracts";
pub const MAX_ARTIFACT_REFS: usize = 512;
pub const MAX_TREE_DIGEST_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_SUPERSEDED: usize = 16;
```

#### `parse_contract_yaml`

```rust
pub fn parse_contract_yaml(text: &str) -> CoreResult<TaskContract>
```

- **Pré:** `text` é UTF-8, já lido sob `read_limited` (≤ 2 MiB).
- **Pós:** `TaskContract` com `extra` preservando chaves desconhecidas.
- **Erros:** `Config("invalid contract yaml: {e}")` em YAML malformado; `Config("unsupported contract schema version")` se `schemaVersion != 1`.
- **Edge cases:** documento vazio → `Config`; `acceptance: []` → **aceito no parse**, rejeitado em `validate_contract` (parse e validação são separados, como `parse_dag_yaml` vs `validate_dag`); YAML com alias/anchor → resolvido por `serde_yaml`; chave duplicada → erro do parser.

#### `validate_contract`

```rust
pub fn validate_contract(
    c: &TaskContract,
    file_stem: &str,
    dag_task_ids: &BTreeSet<String>,
) -> Vec<ContractViolation>
```

- **Pós:** vetor **determinístico** (ordem: campo declarado, depois índice), vazio ⟺ contrato válido. Não retorna `Err` — acumula, para o preflight reportar tudo de uma vez.
- **Violações enumeradas:** `TaskIdUnsafe`, `TaskIdNotInDag`, `TaskIdFileMismatch`, `GoalEmpty`, `GoalTooLong`, `AllowedPathsEmpty`, `AllowedPathsTooMany`, `GlobInvalid{index,message}`, `PathNotRelative{index}`, `AcceptanceEmpty`, `AcceptanceIdDuplicate{id}`, `AcceptanceIdMalformed{id}`, `AcceptanceAssertionEmpty{index}`, `RequiredGatesEmpty`, `RequiredGatesDuplicate{aspect}`, `RepoAliasMalformed{alias}`, `OpenItemsRequiredForFollowUp`.
- **Edge cases:** `allowedPaths: ["**"]` → válido mas sinalizado `AllowedPathsTooBroad` como **warning** (não bloqueia); `taskId` presente no contrato mas ausente do DAG → `TaskIdNotInDag`; arquivo `task-004.yaml` com `taskId: task-005` → `TaskIdFileMismatch`.

#### Paths

```rust
pub fn contract_dir_for_dag(dag_rel: &SafeRelativePath) -> CoreResult<SafeRelativePath>
pub fn contract_rel(dag_rel: &SafeRelativePath, task_id: &str) -> CoreResult<SafeRelativePath>
pub fn handoff_rel(task_id: &str) -> CoreResult<SafeRelativePath>
```

- **Pré:** `task_id` passa `task_id_is_path_safe`; caso contrário `InvalidInput("unsafe handoff task id: {id}")`.
- **Pós:** path relativo POSIX confinado. A-04 aplicado: `parent(dag_rel)/contracts`. DAG na raiz do projeto (`dare-dag.yaml` sem diretório) → `contracts/`.
- **Edge cases:** `dag_rel == "."` → `InvalidInput`; `task_id` com `/`, `..`, `-` inicial ou vazio → `InvalidInput` (regras de `task_id_is_path_safe`).

#### `load_contract`

```rust
pub fn load_contract(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    task_id: &str,
) -> CoreResult<Option<TaskContract>>
```

- **Pós:** `Ok(None)` quando o arquivo **não existe** — ausência é estado legítimo, não erro (D-06). `Ok(Some(_))` quando existe e parseia.
- **Erros:** `Config` em YAML inválido ou schema não suportado; `Config("contract file exceeds size limit")` acima de 2 MiB; `InvalidInput` em path escape.
- **Edge cases:** diretório `contracts/` inexistente → `Ok(None)`; arquivo existe mas é diretório → `Io`; symlink apontando fora da raiz → `InvalidInput` via `ProjectRoot::resolve`.

#### Digests

```rust
pub fn material_digest(c: &TaskContract) -> CoreResult<String>
pub fn full_digest(c: &TaskContract)     -> CoreResult<String>
pub fn digest_bytes(data: &[u8]) -> String
pub fn digest_file(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<String>
pub fn digest_tree(root: &ProjectRoot, globs: &[String]) -> CoreResult<TreeDigest>
```

**Algoritmo de `material_digest`** (determinístico, idêntico em Linux/macOS/Windows):

1. Projetar o contrato no conjunto **fixo** de chaves materiais: `schemaVersion`, `taskId`, `goal`, `scope`, `acceptance`, `verification`, `exitCriteria`, `startingContext`. Chaves fora disso (incluindo `extra`) são descartadas.
2. `serde_json::to_value` da projeção.
3. `to_canonical_json_string` ([`lib.rs:35`](../../../crates/dare-core/src/lib.rs#L35)) — ordena chaves recursivamente.
4. `hex(sha256(bytes))`, lowercase, 64 chars.

`full_digest`: passos 2–4 sobre o contrato inteiro, `extra` incluído.

**`digest_file`:** SHA-256 em streaming, blocos de 64 KiB — não carrega o arquivo em memória. Arquivo ausente → `NotFound`. Arquivo > `MAX_TREE_DIGEST_BYTES` → `Config("artifact exceeds digest size limit")`.

**`digest_tree`:** expande `globs` com `globset`, coleta arquivos regulares, **ordena por path POSIX**, concatena `"{path}\n{file_digest}\n"` e hasheia o resultado. Retorna `TreeDigest { digest, file_count, total_bytes }`.
- **Edge cases:** zero matches → `digest` do string vazio e `file_count == 0` (determinístico, não erro); `file_count > MAX_ARTIFACT_REFS` ou `total_bytes > MAX_TREE_DIGEST_BYTES` → `Config` mapeado para `HANDOFF_SIZE_LIMIT`; symlinks **não** são seguidos (contam como entrada e são ignorados no digest, registrados em `TreeDigest.skipped_symlinks`); ordem independente da ordem de `read_dir` do SO.

#### Política e decisão

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceState { Valid, Missing, Invalid, Stale, NotApplicable, Insufficient }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalCode {
    StrictNoContracts, ContractMissing, ReceiptMissing, ReceiptInvalid,
    ContractDigestMismatch, ArtifactDigestMismatch, VerificationDigestMismatch,
    GateNotSatisfied, VerdictNotPassed, NotFinal, RefEscape, SizeLimit,
    RevisionUnattested,
}

impl RefusalCode {
    /// Wire string estável. Ex.: `RefusalCode::ReceiptMissing` → "HANDOFF_RECEIPT_MISSING".
    pub fn as_str(self) -> &'static str;
    pub fn parse(s: &str) -> Option<Self>;
}

#[derive(Debug, Clone)]
pub struct HandoffDecision {
    pub parent_id: String,
    pub state: EvidenceState,
    pub refusal: Option<RefusalCode>,
    pub detail: String,              // já redigido
    pub imported_digest: Option<String>,
    pub deliverable: Option<String>, // Some ⟺ state == Valid
}

#[derive(Debug, Clone)]
pub struct HandoffPolicy {
    pub strict: bool,
    pub require_attested_revision: bool,
    pub require_contracts: bool,
    pub required_kind: HandoffKind,
}
```

- **Invariante de `HandoffDecision`:** `state == Valid` ⟺ `refusal.is_none()` ⟺ `deliverable.is_some()`. Teste de propriedade garante os três juntos.
- `as_str` / `parse` são round-trip exato para os 13 variantes — teste de estabilidade espelhando `exit_code_mapping_is_stable`.

### 3.2 `dare-verify::handoff`

#### `gates_satisfied`

```rust
pub fn gates_satisfied(
    report: &VerificationReport,
    required: &[GateAspect],
) -> Result<(), Vec<GateAspect>>
```

- **Pré:** `report` lido do disco via `load_verification`.
- **Pós:** `Ok(())` ⟺ para **todo** `a ∈ required` existe `step ∈ report.aspects` com `step.aspect == a`, `step.exit_code == 0` e `!step.timed_out`. Caso contrário `Err(faltantes)`, ordenado por `GateAspect` declarada.
- **Edge cases:** `report.ok == true` mas um gate exigido ausente dos `aspects` → `Err` (não se confia no agregado); `aspects` com duplicata do mesmo gate → basta **uma** aprovada; `required` vazio → `Ok(())`, mas `validate_contract` já rejeita contrato assim.

#### `receipt_from_verification`

```rust
pub fn receipt_from_verification(
    root: &ProjectRoot,
    req: &ReceiptRequest<'_>,
) -> CoreResult<VerificationReceipt>

pub struct ReceiptRequest<'a> {
    pub task_id: &'a str,
    pub contract: &'a TaskContract,
    pub revision: &'a SourceRevision,
    pub created_at: &'a str,   // Clock injetado pelo chamador (padrão de verification_from_ralph)
    pub driver: &'a str,
}
```

- **Pré:** `.dare/verification/<task_id>.json` **existe** e `ok == true`. O relatório é lido do disco, **não** recebido como parâmetro — é o que impede prova circular (RF-03 / §7 A04 do DESIGN).
- **Pós:** recibo com `verdict: Passed`, `verificationDigest` do arquivo canônico, `contractDigest`/`contractFullDigest` recomputados, `artifactRefs`/`artifactDigests` conforme A-06.
- **Erros:** `NotFound` se o relatório não existe → o chamador traduz para `verdict: Insufficient` sem emitir recibo; `Config` se `report.ok == false` (`"refusing to emit receipt for failed verification"`); `Config` nos caps de tamanho.
- **Edge cases:** relatório existe mas `timed_out == true` → `report.ok` é `false`, mesmo caminho; `artifactRefs` vazio com `state == unattested` → recibo **é** emitido (a insuficiência é avaliada no consumo, pela política, não na emissão); `owner` do contrato passa por `redact`.

#### `evaluate_parent`

```rust
pub fn evaluate_parent(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    parent_id: &str,
    policy: &HandoffPolicy,
) -> HandoffDecision
```

Não retorna `Result`: **toda** falha é um `HandoffDecision` com `refusal`, porque a decisão é o produto. Ordem de avaliação fixa — o primeiro `Err` ganha, e a ordem é parte do contrato testável:

| # | Checagem | Recusa |
|---|---|---|
| 1 | Handoff existe em `.dare/handoff/<id>.json` | `ReceiptMissing` |
| 2 | Desserializa, `schemaVersion == 1` | `ReceiptInvalid` |
| 3 | Todo path do recibo é `SafeRelativePath` confinado | `RefEscape` |
| 4 | Contrato existe (se `policy.require_contracts`) | `ContractMissing` |
| 5 | `material_digest(contrato_em_disco) == receipt.contractDigest` | `ContractDigestMismatch` |
| 6 | `digest_file(verificationRef) == receipt.verificationDigest` | `VerificationDigestMismatch` |
| 7 | `gates_satisfied(report, contrato.requiredGates)` | `GateNotSatisfied` |
| 8 | `receipt.verdict == Passed` | `VerdictNotPassed` |
| 9 | Cada `artifactDigest` bate com o arquivo **agora** | `ArtifactDigestMismatch` |
| 10 | `sourceRevision.state != Unattested` (se `require_attested_revision`) | `RevisionUnattested` |
| 11 | `handoff.kind == policy.required_kind` | `NotFinal` |
| 12 | Caps de tamanho em qualquer leitura | `SizeLimit` |

- **TOCTOU:** passos 5, 6 e 9 releem o disco **neste momento**, não usam cache. Em `--agent`, a composição acontece dentro do loop de tentativas ([`execute_agent.rs:259`](../../../crates/dare-cli/src/commands/execute_agent.rs#L259)), então a revalidação ocorre por tentativa.
- **`detail`** sempre passa por `redact` antes de sair.
- **Edge cases:** handoff presente e contrato ausente com `require_contracts == false` → segue, pulando 5; pai com `status != DONE` → o chamador nem chega aqui (filtro de `compose_task_prompt`); `artifactRefs` e `artifactDigests` com tamanhos diferentes → `ReceiptInvalid`.

#### I/O do sidecar

```rust
pub fn write_handoff(root: &ProjectRoot, record: &HandoffRecord) -> CoreResult<()>
pub fn load_handoff(root: &ProjectRoot, task_id: &str) -> CoreResult<HandoffRecord>
pub fn supersede_handoff(root: &ProjectRoot, task_id: &str, reason: &str, at: &str) -> CoreResult<()>
```

- `write_handoff`: valida `task_id_is_path_safe`, aplica `redact` em `deliverable`/`nextSteps`/`openItems`/`detail`, serializa com `to_canonical_json_string`, grava com `atomic_write` sob `FileLock` em `.dare/handoff/<id>.json.darelock`. Concorrência: **fail-fast**, igual a `.dare/state.json` (`Io("file lock held")`), sem fila.
- `supersede_handoff`: lê o registro, move o `receipt` atual para `superseded[]` com `reason`, trunca FIFO em `MAX_SUPERSEDED`, regrava atomicamente. Chamado no `Reset` e na reexecução — é o que cumpre RF-03 e o caso 5 do aceite. Registro ausente → `Ok(())` (idempotente).

### 3.3 `dare-dag::execution` — composição

```rust
pub trait ParentEvidence {
    fn decide(&self, parent_id: &str) -> HandoffDecision;
}

pub struct ComposedPrompt {
    pub prompt: String,
    pub imported: Vec<String>,          // ids aceitos, ordem lexico
    pub refused: Vec<HandoffDecision>,  // ids recusados, ordem lexico
}

pub fn compose_task_prompt_verified(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    task_id: &str,
    ev: &dyn ParentEvidence,
) -> Result<ComposedPrompt, DagGraphError>
```

- **Pré:** `task_id` existe no DAG, senão `InvalidDag { message: "task not found: {id}" }` (mensagem **idêntica** à atual).
- **Pós:** mesma estrutura de prompt da função atual — `subtask_prompt`, depois `## Upstream context`, depois `### From parent: {id} — {title}` por pai aceito, com `unicode_tail(limit)` aplicado. **Pais recusados não aparecem no prompt.**
- **Invariante de compatibilidade:** se `ev.decide` devolve `Valid` com o `deliverable` igual ao `state.tasks[id].output` para todo pai `DONE`, a `prompt` é **byte-idêntica** à de `compose_task_prompt`. Isso é um teste, não uma intenção.
- **Edge cases:** zero pais → `prompt == subtask_prompt`, `imported` e `refused` vazios (sem cabeçalho `## Upstream context`); todos recusados → idem, com `refused` populado; `deliverable` vazio após `trim` → pai omitido, como hoje; DAG `Legacy` → `subtask_prompt` é `""`, comportamento preservado.
- **`compose_task_prompt` não muda.** `build_next_report` continua chamando a versão antiga quando `strict == false` (A-03).

### 3.4 `dare-cli` — orquestração

```rust
// commands/handoff_policy.rs
pub fn handoff_policy_from_cfg(cfg: &DareConfig, cli: &HandoffFlags) -> HandoffPolicy

// commands/handoff_preflight.rs
pub fn run_handoff_preflight(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    doc: &DagDocument,
    policy: &HandoffPolicy,
) -> CoreResult<PreflightReport>

pub struct PreflightReport {
    pub schema_version: u32,     // 1
    pub strict: bool,
    pub tasks_total: u32,
    pub tasks_with_contract: u32,
    pub violations: Vec<(String, ContractViolation)>,
}
```

`run_handoff_preflight`:
- **Pré:** só é chamada quando `policy.strict == true`. Com `strict == false` é um no-op não invocado — zero custo no caminho default.
- **Pós:** `Ok(report)` com contagens e violações acumuladas de todos os contratos presentes.
- **Erros:** `GuardFail("handoff preflight failed: strict mode enabled but no task contract found under {dir}")` quando `tasks_with_contract == 0` → exit **6**, código `HANDOFF_STRICT_NO_CONTRACTS`. `GuardFail` também quando há violações bloqueantes, listando até 10 e o total.
- **Edge cases:** diretório `contracts/` inexistente → mesma recusa do caso zero contratos (mensagem nomeia o diretório esperado); 1 de 11 tasks com contrato → preflight **passa**, e as outras 10 caem em `HANDOFF_CONTRACT_MISSING` no consumo (é a distinção exigida por D-06.1); DAG vazio → `tasks_total == 0`, recusa com a mesma mensagem.

---

## 4. Superfície CLI

### 4.1 Flags novas em `dare execute`

```rust
/// Enforce verified handoffs before importing parent context (config `handoff.strict`).
#[arg(long = "strict-handoff", action = clap::ArgAction::SetTrue,
      overrides_with = "no_strict_handoff")]
strict_handoff: bool,

/// Disable strict handoff enforcement for this run.
#[arg(long = "no-strict-handoff", action = clap::ArgAction::SetTrue)]
no_strict_handoff: bool,
```

Precedência (igual a `--verify` / `--no-verify`): flag CLI > `dare.config.json#handoff.strict` > `false`. Aplicam-se a `--next`, `--agent` e `--complete`. Nenhuma flag existente muda de semântica; nenhum `conflicts_with` novo além do par mútuo.

### 4.2 Exit codes

| Situação | `strict = false` | `strict = true` |
|---|---|---|
| Handoff válido | 0 | 0 |
| Recusa de handoff | **0** + aviso em stderr | **6** (`GuardFail`) |
| Preflight sem contratos | n/a (não roda) | **6** (`GuardFail`) |
| Gate Ralph reprovado | 1 (inalterado) | 1 (inalterado) |

O exit 6 em `dare execute` é **Classe B** — novo para o comando, não para a tabela global ([`error.rs:31`](../../../crates/dare-core/src/error.rs#L31)), e inalcançável no default.

### 4.3 Saída JSON

Envelope global **intocado**. Campos **adicionados** dentro de `data`:

`dare execute --next --json` → cada item de `ready` ganha:

```json
{
  "handoff": {
    "strict": false,
    "imported": ["task-003"],
    "refused": [
      { "parentId": "task-002", "state": "missing", "code": "HANDOFF_RECEIPT_MISSING",
        "detail": "no handoff record at .dare/handoff/task-002.json" }
    ]
  }
}
```

`dare execute --complete --json` → `data` ganha `"handoffPath": ".dare/handoff/<id>.json"` e `"receipt": { … }`, somando-se aos campos atuais de `complete_json` ([`execute.rs:686`](../../../crates/dare-cli/src/commands/execute.rs#L686)).

Em recusa estrita, o envelope de erro traz `error.kind == "GuardFail"` e a mensagem; o código estável vai na mensagem **e** no `data` do relatório quando houver. Campos novos são **aditivos** — política de disco e JSON ([`disk-and-json-policy.md`](../../../docs/compatibility/disk-and-json-policy.md)): "novo campo opcional permitido com default seguro".

### 4.4 Saída humana

```
warning: handoff from task-002 not verified (HANDOFF_RECEIPT_MISSING); imported anyway
         (compatibility mode — this is not verification; enable handoff.strict to block)
```

Em estrito:

```
error: handoff refused: task-002 (HANDOFF_RECEIPT_MISSING)
       no handoff record at .dare/handoff/task-002.json
```

Mudança material de contrato (RF-02), nomeando o caminho de recuperação:

```
error: contract changed after task start: task-004 (HANDOFF_CONTRACT_DIGEST_MISMATCH)
       material fields (goal/scope/acceptance/exitCriteria) differ from the recorded digest
       to adopt the new contract: dare execute --reset task-004
```

Todas as mensagens passam por `redact` e `strip_ansi` conforme o renderer já faz ([`output.rs`](../../../crates/dare-cli/src/output.rs)).

---

## 5. Segurança

Mapeamento OWASP completo está em [DESIGN §7](DESIGN.md). O que o blueprint fixa como **implementação**:

| Controle | Mecanismo concreto |
|---|---|
| Confinamento de path | **Toda** referência de recibo/contrato passa por `SafeRelativePath::new` + `ProjectRoot::resolve`. `resolve` já nega escape por symlink via `verify_within_root` + `canonicalize` ([`path.rs:148`](../../../crates/dare-core/src/path.rs#L148)) — o controle é reuso, não código novo. `PATH_ESCAPE_MSG` → `RefusalCode::RefEscape` |
| Symlinks em `digest_tree` | **Não** seguidos. Contados em `TreeDigest.skipped_symlinks`, fora do digest |
| Segredos | `redact` em `deliverable`, `nextSteps`, `openItems`, `owner`, `detail` e toda mensagem de recusa — mesmo tratamento que `write_verification` dá a `stdout_tail`/`stderr_tail` |
| Prompt injection | Pai recusado **não entra** no prompt em estrito. O `deliverable` é resumo limitado por `contextBounds`, nunca log completo |
| Prova circular | `receipt_from_verification` **lê** `.dare/verification/<id>.json` do disco; não aceita `RalphReport` como parâmetro. `report.ok == false` → `Config`, sem recibo |
| OOM / arquivo malicioso | `read_limited` (2 MiB) em contrato e sidecar; `MAX_ARTIFACT_REFS` 512; `MAX_TREE_DIGEST_BYTES` 64 MiB; digest em streaming de 64 KiB |
| TOCTOU | Passos 5/6/9 de `evaluate_parent` releem disco no momento do consumo; em `--agent`, por tentativa |
| Concorrência | `FileLock` fail-fast no sidecar, padrão de `.dare/state.json` |
| Supply chain | Zero dependência nova (A-08). `cargo audit` + `cargo deny check` no gate |
| Sem rede | Nenhuma chamada de rede introduzida. `SafeCommand` sobre `git` com timeout 5 s e `stdout_limit`, padrão de [`git.rs:33`](../../../crates/dare-project/src/git.rs#L33) |

---

## 6. Arquivos a criar e alterar

```
crates/dare-contracts/
├── Cargo.toml                    ~ + sha2 = { workspace = true }
└── src/
    ├── lib.rs                    ~ + pub mod handoff; re-exports
    └── handoff.rs                + NOVO  tipos, digests, paths, política pura

crates/dare-verify/
└── src/
    ├── lib.rs                    ~ + pub mod handoff; pub mod handoff_store;
    ├── handoff.rs                + NOVO  receipt, gates, evaluate_parent
    └── handoff_store.rs          + NOVO  I/O do sidecar, supersede (arquivo próprio: §7)

crates/dare-dag/
└── src/
    ├── lib.rs                    ~ + re-export ParentEvidence, ComposedPrompt
    └── execution.rs              ~ + compose_task_prompt_verified (fn atual intocada)

crates/dare-project/
└── src/git.rs                    ~ + head_sha(), worktree_is_dirty(), changed_files()

crates/dare-cli/
└── src/
    ├── main.rs                   ~ + --strict-handoff / --no-strict-handoff
    └── commands/
        ├── execute.rs            ~ emite recibo no complete; supersede no reset
        ├── execute_agent.rs      ~ usa compose_task_prompt_verified quando strict
        ├── handoff_policy.rs     + NOVO  resolução config × flags
        └── handoff_preflight.rs  + NOVO  D-06.1

tests/
├── golden/handoff_*.rs           + NOVO  goldens next/complete, strict on/off
└── security/handoff_*.rs         + NOVO  escape, symlink, segredo, caps

docs/
├── DECISION-LOG.md               ~ + DEC-059
└── compatibility/
    ├── handoff-contracts.md      + NOVO  contrato de compatibilidade do ciclo
    └── classification-matrix.md  ~ + linha Classe B do exit 6
```

---

## 7. Plano de execução

11 tasks em **7 ranks** (0–6), largura máxima de 3. Ranks confirmados por `dare dag viz` em [`dag-graph.mmd`](dag-graph.mmd) — longest-path, não atribuídos à mão.

`complexity` usa os valores que o validador aceita: `LOW` | `MED` | `HIGH`, case-sensitive ([`COMPLEXITY_ALLOWED`](../../../crates/dare-dag/src/report.rs#L8)). Não existe `MEDIUM`.

| Task | Título | `depends_on` | Rank | Complexity |
|---|---|---|---|---|
| `task-001` | Golden baseline de `execute --next` / `--complete` **antes** de qualquer mudança | — | 0 | LOW |
| `task-002` | `dare-contracts::handoff` — tipos, parse, validate, paths | — | 0 | HIGH |
| `task-004` | `dare-project::git` — `head_sha`, `worktree_is_dirty`, `changed_files` | — | 0 | MED |
| `task-003` | `dare-contracts::handoff` — digests (material, full, file, tree) + caps | `task-002` | 1 | MED |
| `task-005` | `dare-verify::handoff` — `gates_satisfied` + `receipt_from_verification` | `task-003`, `task-004` | 2 | HIGH |
| `task-006` | `dare-verify::handoff_store` — I/O do sidecar + `supersede_handoff` | `task-003` | 2 | MED |
| `task-008` | `dare-dag` — `ParentEvidence` + `compose_task_prompt_verified` | `task-003` | 2 | MED |
| `task-007` | `dare-verify::handoff` — `evaluate_parent` (12 checagens, ordem fixa) | `task-005`, `task-006` | 3 | HIGH |
| `task-009` | `dare-cli` — política, flags, preflight D-06.1 | `task-007`, `task-008` | 4 | HIGH |
| `task-010` | `dare-cli` — emissão no `--complete`, supersede no `--reset`, JSON | `task-009` | 5 | HIGH |
| `task-011` | Testes negativos, segurança, round-trip npm, docs + DEC-059 | `task-010`, `task-001` | 6 | HIGH |

**`task-001` é rank 0 e obrigatória antes de qualquer mudança de comportamento** — é o baseline que torna O-04 verificável em vez de afirmado. Sem ele, o caso 11 do aceite não tem referência de comparação.

**Refinamento de §6 decidido na fase de tasks:** `dare-verify` ganha **dois** arquivos, não um. `handoff.rs` (`task-005` + `task-007`) e `handoff_store.rs` (`task-006`). Sem isso, `task-005` e `task-006` — mesmo rank, paralelizáveis — editariam o mesmo arquivo e conflitariam em worktrees. Cada uma é dona de um arquivo; ambas acrescentam um `pub mod` de uma linha a `lib.rs`.

### Rank 0 — fundação e baseline (3 em paralelo)
`task-001`, `task-002`, `task-004`. Nenhuma toca comportamento observável.

### Ranks 1–2 — primitivas
`task-003`, depois `task-005` / `task-006` / `task-008` em paralelo. Tudo library-only, testável sem CLI.

### Rank 3 — decisão
`task-007`. A ordem das 12 checagens é parte do contrato; cada uma ganha teste negativo próprio.

### Ranks 4–6 — integração e fecho
`task-009` → `task-010` → `task-011`. É onde o comportamento observável muda, sempre atrás de `strict`.

A cauda é sequencial por duas razões reais, não por acoplamento falso: `task-010` edita o mesmo `commands/execute.rs` que `task-009` (serialização por propriedade de arquivo, evita conflito de merge), e `task-011` precisa da integração completa para comparar contra o baseline de `task-001`. A largura do ciclo está nos ranks 0–2.

---

## 8. Matriz de testes

| # | Caso do aceite | Teste | Onde |
|---|---|---|---|
| 1 | Positivo | `handoff_accepted_full_chain` | `dare-verify` + golden |
| 2 | Sem evidência | `strict_blocks_missing_receipt`, `compat_warns_missing_receipt` | `dare-verify`, golden |
| 3 | Adulteração | `tamper_contract`, `tamper_artifact`, `tamper_verification` (3 testes) | `dare-verify` |
| 4 | Gate reprovado | `no_receipt_when_ralph_fails`, `gates_satisfied_rejects_missing_aspect` | `dare-verify` |
| 5 | Reexecução | `reset_supersedes_receipt`, `superseded_fifo_cap` | `dare-verify` |
| 6 | Pais múltiplos | `multi_parent_partial_refusal` | `dare-dag` (com `ParentEvidence` falso) |
| 7 | Segurança | `ref_escape_absolute`, `ref_escape_dotdot`, `symlink_not_followed`, `secret_redacted_in_handoff` | `tests/security` |
| 8 | Compatibilidade | `strict_off_byte_identical` (vs. golden de `task-001`), `npm_roundtrip_preserves_sidecar`, 27 goldens de `init` | `tests/golden` |
| 9 | Preflight | `preflight_zero_contracts_legacy`, `preflight_zero_contracts_v21`, `preflight_passes_with_one_contract` | `dare-cli` |
| 10 | Recuperação por reset | `material_change_refused_names_reset`, `reset_then_start_records_new_digest` | `dare-cli` |
| 11 | Exit code | `strict_refusal_exits_6`, `compat_exits_0` | `tests/golden` |

Testes de propriedade (`proptest`, já dev-dep de `dare-dag`): invariante de `HandoffDecision` (3 campos coerentes); `material_digest` estável sob reordenação de chaves YAML; `digest_tree` independente da ordem de `read_dir`; round-trip `RefusalCode::as_str`/`parse` nos 13 variantes.

---

## 9. Compatibilidade

| Item | Classe | Nota |
|---|---|---|
| Campos novos em `data` do `--json` | **A** | Aditivos, política de disco e JSON permite |
| `compose_task_prompt` | **A** | Assinatura e saída inalteradas |
| `.dare/state.json` | **A** | Nenhum campo novo; sidecar é arquivo separado |
| `dare.config.json` | **A** | `handoff` lido de `extra`; `init` não escreve |
| Exit 6 em `dare execute` sob `strict` | **B** | Novo para o comando, não para a tabela global; inalcançável no default |
| Aviso em stderr no modo compat | **B** | stdout intocado; stderr ganha linha nova quando há recusa |
| `--strict-handoff` / `--no-strict-handoff` | **B** | Flags novas, default preserva comportamento |
| Sidecar não reconhecido pelo npm | **B** | Documentado: npm compõe contexto sem verificação, como hoje |

Nada de Classe C ou D. Round-trip Rust → npm → Rust coberto por `npm_roundtrip_preserves_sidecar`.

---

## 10. Ralph Loop

```bash
cargo fmt --all                                          # antes de commitar
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
cargo deny check
```

Falha em qualquer gate impede DONE. Como A-08 não adiciona dependência, `cargo audit` e `cargo deny` devem sair idênticos ao baseline — divergência aí é sinal de erro no manifesto.

---

## 11. Próximas etapas

1. **Revisar e aprovar este Blueprint.**
2. `/dare-tasks DARE/cycles/rust-verified-task-contracts-012/BLUEPRINT.md` — gera `TASKS.md`, `dare-dag.yaml` e `EXECUTION/task-001..011.md`, com os contratos de §2.1 por task.
3. `dare validate DARE/cycles/rust-verified-task-contracts-012/dare-dag.yaml`.
4. `/dare-execute` task a task, Ralph Loop completo por task.
5. `REGRESSION.md`, `PROOF.md`, `docs/compatibility/handoff-contracts.md`, `DEC-059`.
6. PR em Draft — **sem merge sem autorização humana específica**.
