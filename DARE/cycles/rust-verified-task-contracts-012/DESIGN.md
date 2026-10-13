# DESIGN — Verified Task Contracts & Handoffs

> **Ciclo:** `rust-verified-task-contracts-012` (12º ciclo; PR lógico 1 de 4)
> **Entrada:** `IDEIA-01-Verified-Task-Contracts-Handoffs.md` (v2.0, 2026-10-13) — tratada como entrada, **não** como design aprovado
> **Baseline Rust:** `main` @ `da5f571` — workspace Cargo 4.0.0, MSRV 1.88.0
> **Baseline npm:** `@dewtech/dare-cli@3.18.1` — `dare-method` @ `274c3b2`
> **Decisão a registrar no fecho:** DEC-059
> **Status:** aguardando aprovação humana (fase R do DARE)

---

## 1. Problema

O runtime de DAG já tem tudo o que precisa para *produzir* prova e nada que a obrigue a ser *consumida*. A fronteira entre duas tasks é o ponto cego.

**Verificado na `main` em 2026-10-13:**

| Local | O que faz hoje | Lacuna |
|---|---|---|
| [`crates/dare-dag/src/execution.rs:115`](../../../crates/dare-dag/src/execution.rs#L115) | `compose_task_prompt` percorre `depends_on` ordenado, pega `state.tasks[dep].output` de todo pai com `status == "DONE"`, corta pelo `limits.parentContextChars` e injeta sob `## Upstream context` | Nenhuma checagem de integridade, origem ou evidência no ponto de consumo. `DONE` + string arbitrária = contexto confiável |
| [`crates/dare-verify/src/verification.rs:17`](../../../crates/dare-verify/src/verification.rs#L17) | `VerificationReport { schemaVersion, taskId, ok, timedOut, stack, aspects, updatedAt }`, gravado redigido e canônico em `.dare/verification/<task-id>.json` | Sem `contractDigest`, sem `artifactDigests`, sem `sourceRevision`. O relatório prova *que os gates rodaram*, não *sobre qual código nem sobre qual contrato* |
| [`crates/dare-cli/src/commands/execute.rs:252`](../../../crates/dare-cli/src/commands/execute.rs#L252) | `run_complete_inner`: Ralph → `write_verification` → advanced verify → `Transition::Complete { output }` | O `output` é texto livre truncado por `limits.taskOutputChars`. Nunca é ligado ao relatório que acabou de ser escrito |
| [`crates/dare-contracts/src/dag.rs:44`](../../../crates/dare-contracts/src/dag.rs#L44) | `DagTask { id, title, dependsOn, complexity, subtaskPrompt, specFile, #[serde(flatten)] extra }` | Goal / Scope / Acceptance / Exit Criteria não existem como campos validáveis. `subtaskPrompt` é prosa |
| [`crates/dare-contracts/src/state.rs:26`](../../../crates/dare-contracts/src/state.rs#L26) | `TaskRuntimeState` preserva campos desconhecidos via `#[serde(flatten)] extra` | O npm `packages/cli/src/dag-runner/state-store.ts::saveState` reconstrói `.dare/state.json` com campos conhecidos. **Extensão em `state.json` é perdida no round-trip** |
| [`crates/dare-project/src/git.rs`](../../../crates/dare-project/src/git.rs) | `find_git_root` — descobre o toplevel, degrada para `None` sem git | Não existe helper de HEAD SHA nem de estado clean/dirty |
| [`crates/dare-core/src/fs/backup.rs:9`](../../../crates/dare-core/src/fs/backup.rs#L9) | SHA-256 usado só para derivar nome de diretório de backup | Não há primitiva pública de digest de conteúdo de arquivo |

**Consequência concreta:** hoje, `dare execute --complete task-a --output "feito"` em um projeto onde o Ralph passou e o código foi depois alterado produz um `task-b` cujo prompt contém `"feito"` como fato. Alterar um byte do artefato de `task-a` após a verificação não muda nada observável no consumo. O status `DONE` é a única prova exigida, e ele é uma declaração, não uma evidência.

A oportunidade **não** é refazer DAG, Ralph ou `VerificationReport`. É fechar a cadeia *contrato → execução → evidência → recibo → consumo*.

---

## 2. Objetivo

Ao fim do ciclo, uma execução consegue responder, com artefato em disco: **"a task B importou o resultado de A porque o contrato de A, seus artefatos, sua verificação e a revisão de origem foram validados no momento do consumo"** — e, quando recusa, **por qual motivo, com código estável**.

| ID | Objetivo | Critério de sucesso mensurável |
|---|---|---|
| O-01 | Impedir importação não verificada em modo estrito | 100% dos cenários de fixture com recibo ausente ou inválido são bloqueados com código de recusa |
| O-02 | Preservar proveniência e integridade | Alterar 1 byte de artefato, contrato ou relatório de verificação invalida o handoff |
| O-03 | Permitir reprodução da entrega | 100% dos handoffs aceitos contêm referências de evidência, digests e revisão Git quando disponível |
| O-04 | Preservar usuários existentes | `handoff.strict = false` (default) → saída byte-idêntica à da `main` nos goldens de `dare execute` |
| O-05 | Manter testes previsíveis | Fixtures positivas e negativas verdes em Linux, macOS e Windows no CI |

---

## 3. Decisões desta fase

As pendências que a IDEIA §13 deixou explicitamente para o DESIGN, resolvidas aqui.

### D-01 — Escopo do ciclo: somente os MUST

Entram **RF-01, RF-02, RF-03, RF-04, RF-05, RF-09 e RF-10**.

Ficam para o PR 02 (`Loop Blocks`), com justificativa: **RF-06** (comandos de inspeção) depende do formato estar congelado; **RF-07** (encadeamento multi-pai declarado) e **RF-08** (invalidação por reuso) são refinamentos da política que RF-05 já implementa de forma conservadora; **RF-11** (revisão independente + Human Gate) exige modelo de identidade de runner que este ciclo apenas registra, não decide.

> Nota: o ciclo ainda **implementa** o comportamento seguro para pais múltiplos — RF-05 valida *cada* pai independentemente e recusa individualmente. O que fica para RF-07 é a *declaração* de quais digests foram importados e a semântica configurável de dependência (todos vs. qualquer).

### D-02 — Storage: contrato no Git, evidência em sidecar privado

```
DARE/cycles/<ciclo>/contracts/<task-id>.yaml   ← Task Contract — versionado, revisável em PR
.dare/handoff/<task-id>.json                   ← Receipt + Handoff — sidecar privado, NÃO versionado
.dare/verification/<task-id>.json              ← já existe, inalterado neste ciclo
```

Racional:

- O contrato é **intenção humana**: precisa de diff em PR, autoria por commit e revisão. YAML, para combinar com `dare-dag.yaml`.
- O recibo é **evidência de execução local**: contém paths, digests e timestamps da máquina que executou. No Git ele poluiria todo diff e arrastaria dados de ambiente. JSON canônico, igual ao `VerificationReport`.
- Ambos ficam **fora** de `.dare/state.json` e **fora** dos campos de `dare-dag.yaml` que o npm regrava — resolve RF-10 por construção.
- `.dare/handoff/` entra no bloco DARE do `.gitignore` gerado pelo `init`. **Sem alterar o contrato do `init`**: o `.dare/` já é ignorado como diretório; nada novo precisa ser escrito lá.

### D-03 — Modo estrito: config + flag CLI, default `false`

```json
{ "handoff": { "strict": false } }
```

Lido de `DareConfig.extra`, seguindo exatamente o precedente de [`verify_enabled_from_cfg`](../../../crates/dare-verify/src/advanced.rs#L135) e `formal_enabled_from_cfg`. Override por `dare execute --strict-handoff` / `--no-strict-handoff` (padrão `overrides_with`, como `--verify` / `--no-verify`).

- **Default `false` (compatibilidade):** comportamento atual preservado byte a byte. Quando uma importação *seria* recusada, emite aviso identificável em stderr e marca o campo de evidência no JSON — **sem** bloquear e **sem** alterar o prompt composto.
- **`true` (estrito):** *fail closed*. Importação recusada aborta a composição com código estável e exit code mapeado.

Compatibilidade **não é** verificação: o aviso diz o que *seria* bloqueado, nunca afirma que o handoff está válido.

### D-04 — Digest prova integridade, não autoria

SHA-256 sobre bytes canônicos prova que o conteúdo corresponde a um digest de referência. **Não** prova identidade, autenticidade nem autoria. O v1 não reivindica assinatura. Se assinatura vier a ser necessária, usa-se a infraestrutura `cosign` já presente em `dare-self` ([DEC-054](../../../docs/DECISION-LOG.md)) com cadeia de confiança explícita — fora deste ciclo.

### D-05 — `Follow-up Handoff` não é `DONE`

Dois tipos de saída, campo `exitCriteria.kind` no contrato e `kind` no handoff:

| Tipo | Semântica | Desbloqueia dependente? |
|---|---|---|
| `verified-deliverable` | Contrato cumprido, gates aprovados, evidência completa | Sim |
| `follow-up-handoff` | Trabalho parcial legítimo, estado e próximo ponto de entrada registrados | **Não**, quando o consumidor exige entrega final |

Em modo estrito, um dependente que exige `verified-deliverable` e encontra `follow-up-handoff` recusa com `HANDOFF_NOT_FINAL`. Em compatibilidade, avisa.

### D-06 — Legacy DAG degrada, não quebra

`DagDocument::Legacy` ([`execution.rs:337`](../../../crates/dare-dag/src/execution.rs#L337)) não tem `subtaskPrompt` nem `specFile`. Isso **não** impede contrato: por D-02 o contrato é um arquivo indexado por `taskId`, então `DARE/cycles/<ciclo>/contracts/task-001.yaml` funciona igual para legacy e para v2.1. A diferença é só que um DAG legacy, hoje, *não tem* contratos — não que não *possa* ter.

Portanto: contrato é **opcional e ausente por default** nesses DAGs, e o estrito recusa em vez de inventar contrato vazio. Nenhuma migração compulsória de DAG v2.1 neste release.

**Preflight de ativação (D-06.1).** Recusar task por task seria correto mas diagnóstico ruim: quem liga `handoff.strict = true` num projeto sem contratos descobre isso no meio do run, uma task por vez. Com o estrito ativo, antes de compor qualquer prompt, conta-se quantas tasks do DAG têm contrato:

| Situação | Código | Natureza |
|---|---|---|
| Estrito ativo e **nenhuma** task tem contrato | `HANDOFF_STRICT_NO_CONTRACTS` | Misconfiguração — falha uma vez, no início |
| Estrito ativo, projeto tem contratos, **este pai** não tem | `HANDOFF_CONTRACT_MISSING` | Lacuna real na task — falha no consumo |

Separar os dois é o que a linha A05 de §7 exige: o usuário precisa distinguir "configurei errado" de "falta evidência aqui". Segue o padrão de [`dare-guard/src/preflight.rs:16`](../../../crates/dare-guard/src/preflight.rs#L16) — `run_preflight` já varre superfícies de controle antes do `--agent` começar e falha fechado com `GuardFail`. Custo de uma checagem, sem comando novo, sem invadir RF-06.

---

## 4. Requisitos funcionais

### RF-01 — Task Contract versionado (MUST)

Representação estruturada, validável e serializável determinística para:

| Campo | Obrigatório | Nota |
|---|---|---|
| `schemaVersion` | sim | `1` |
| `taskId` | sim | Deve existir no DAG e passar por `task_id_is_path_safe` |
| `goal` | sim | Uma frase; o *quê*, não o *como* |
| `scope.allowedPaths` | sim | Globs relativos, confinados à raiz |
| `acceptance[]` | sim | `{ id, assertion }` — ao menos 1 |
| `verification.requiredGates[]` | sim | Subconjunto de `build` \| `test` \| `lint` (aspectos Ralph existentes) |
| `exitCriteria.kind` | sim | `verified-deliverable` \| `follow-up-handoff` |
| `startingContext` | sim | Fonte do contexto inicial; ver RF-09 |
| `taskType`, `owner`, `requirementRefs[]` | não | Campos opcionais planejados, com default seguro |

O contrato **não duplica** o `id` como fonte de verdade: o DAG continua sendo o grafo. O contrato é indexado *por* `taskId` e sua existência é opcional por task. DAGs v2.1 sem contrato continuam executando.

Forma final de armazenamento no YAML (campo opcional em `DagTask.extra` vs. arquivo por task vs. ambos) → **BLUEPRINT**. D-02 já fixa que o arquivo por task é o caminho canônico.

### RF-02 — Fronteira do contrato (MUST)

- Antes de iniciar a execução (transição `Start`), computar e registrar o `contractDigest` — SHA-256 do contrato em serialização canônica.
- A execução **não** pode alterar `goal`, `scope`, `acceptance` ou `exitCriteria` e ainda assim aparentar sucesso: na conclusão, o digest é recomputado e comparado. Divergência → `HANDOFF_CONTRACT_DIGEST_MISMATCH`.
- Mudança **material** (qualquer um dos 4 campos acima) exige nova revisão do contrato ou nova task — nunca mutação silenciosa.
- Mudança **não material** (`owner`, `requirementRefs`, comentários) deixa trilha auditável: digest anterior e novo registrados no histórico do recibo.
- Política de aprovação humana para mudança material: ver §7 (Segurança). O *mecanismo* de gate fica em RF-11 / PR 02; neste ciclo a mudança material é **recusada**, não aprovada.

**Caminho de recuperação sancionado.** Recusar não é beco sem saída, e o escape hatch já existe: `Transition::Reset` ([`state.rs:164`](../../../crates/dare-dag/src/state.rs#L164)) vai de `Running` / `Done` / `Failed` / `Skipped` para `Pending`, limpa `output` e `error` e **preserva `attempts`**. O fluxo para mudança material legítima é:

1. Corrigir o contrato em `DARE/cycles/<ciclo>/contracts/<task-id>.yaml`.
2. `dare execute --reset <task-id>` — ação deliberada de um humano.
3. Novo `Start` regrava o `contractDigest`; o recibo anterior fica marcado `superseded` (RF-03), preservado como histórico e nunca reutilizável como válido.

Um humano rodando `--reset` **é** o Human Gate deste ciclo — um gate que já existe, já é testado e não depende de modelo de identidade de runner. A mensagem de recusa por mudança material deve nomear esse comando explicitamente, para que a recusa seja acionável e não apenas um bloqueio. O que RF-11 acrescenta no PR 02 é a outra metade: autor ≠ verificador, que exige identidade de runner real — o campo `producer` do RF-03 é fraco demais para sustentar essa afirmação.

### RF-03 — Verification Receipt (MUST)

Emitido **após** verificação real, derivado da evidência já produzida — nunca de um booleano.

| Campo | Fonte |
|---|---|
| `schemaVersion` | `1` |
| `taskId` | DAG |
| `contractDigest` | RF-02 |
| `sourceRevision` | RF-09 — `attested` \| `unattested` \| `dirty`, nunca inventado |
| `artifactRefs[]` | Paths relativos confinados, derivados de `scope.allowedPaths` |
| `artifactDigests[]` | SHA-256 dos **bytes efetivos** de cada artefato referenciado |
| `verificationRef` | `.dare/verification/<task-id>.json` |
| `verificationDigest` | SHA-256 do relatório canônico lido do disco |
| `verdict` | `passed` \| `failed` \| `insufficient` |
| `createdAt` | `Clock::now_rfc3339()` (`FixedClock` nos goldens) |
| `producer` | Identificação do runner (driver, versão do CLI) |

Regras:

- `status == DONE` **não** é prova. O recibo só existe se o `VerificationReport` em disco for lido, validado e tiver `ok == true`.
- Gate reprovado → **nenhum** recibo de `verified-deliverable` é emitido. O Ralph Loop não é contornado.
- Ausência de commit Git → `sourceRevision.state = "unattested"`, não revisão fabricada.
- Reexecução **não** sobrescreve evidência anterior silenciosamente: recibos antigos são preservados como histórico e marcados `superseded`, nunca reutilizados como válidos sem nova verificação.

### RF-04 — Handoff explícito (MUST)

Registro de entrega de A para B, no mesmo sidecar do recibo:

- `kind`: `verified-deliverable` \| `follow-up-handoff` (D-05)
- `deliverable`: o resultado importável — **resumo limitado**, não cópia de log
- `receiptRef` / `receiptDigest`: ligação com RF-03
- `contextBounds`: limite efetivo aplicado (`limits.parentContextChars`)
- `nextSteps[]` e `openItems[]`: pendências identificadas

Um `follow-up-handoff` registra trabalho parcial e ponto de entrada. **Não** implica contrato cumprido.

### RF-05 — Verificação no consumidor (MUST)

O ponto de mudança é `compose_task_prompt`. Antes de incorporar o resultado de A no prompt de B, **para cada pai independentemente**:

1. Recibo e handoff existem e desserializam no `schemaVersion` suportado.
2. `contractDigest` do recibo bate com o contrato em disco **agora** (revalidação imediatamente antes do consumo — mitiga TOCTOU).
3. `verificationDigest` bate com `.dare/verification/<task-id>.json` **agora**.
4. Cada `artifactDigest` bate com os bytes do artefato **agora**.
5. Todos os `verification.requiredGates` do contrato aparecem aprovados nos `aspects` do relatório.
6. Toda referência é relativa, confinada à raiz do projeto, sem symlink traversal nem path absoluto externo.
7. `kind` satisfaz o que o consumidor exige.

Estados de evidência distinguíveis — `valid` \| `missing` \| `invalid` \| `stale` \| `not_applicable` \| `insufficient`.

Códigos de recusa estáveis (conjunto fechado neste ciclo; nomes finais confirmados no BLUEPRINT):

| Código | Causa |
|---|---|
| `HANDOFF_STRICT_NO_CONTRACTS` | Estrito ativo e nenhuma task do DAG tem contrato — preflight, D-06.1 |
| `HANDOFF_CONTRACT_MISSING` | Contrato exigido e ausente nesta task |
| `HANDOFF_RECEIPT_MISSING` | `DONE` sem recibo |
| `HANDOFF_RECEIPT_INVALID` | Recibo malformado ou schema não suportado |
| `HANDOFF_CONTRACT_DIGEST_MISMATCH` | Contrato alterado após a verificação |
| `HANDOFF_ARTIFACT_DIGEST_MISMATCH` | Artefato alterado após a verificação |
| `HANDOFF_VERIFICATION_DIGEST_MISMATCH` | Relatório de verificação alterado |
| `HANDOFF_GATE_NOT_SATISFIED` | Gate exigido pelo contrato não aprovado |
| `HANDOFF_VERDICT_NOT_PASSED` | `verdict != passed` |
| `HANDOFF_NOT_FINAL` | `follow-up-handoff` onde se exige entrega final |
| `HANDOFF_REF_ESCAPE` | Path fora da raiz, absoluto ou symlink traversal |
| `HANDOFF_SIZE_LIMIT` | Artefato de contrato/recibo acima do limite |
| `HANDOFF_REVISION_UNATTESTED` | Política exige revisão atestada e não há |

O envelope global de `--json` (`{ ok, data, correlation_id }` / `{ ok, error: { kind, message } }`, [`output.rs`](../../../crates/dare-cli/src/output.rs)) **não muda**. Os códigos vão em `data`, não em `error.kind`.

**Mapeamento da recusa estrita: `CoreError::GuardFail` → exit 6.**

| Alternativa | Veredito |
|---|---|
| `GuardFail` (6) | **Escolhida.** É literalmente o que é: um gate de segurança que falhou fechado numa checagem de integridade. `dare-guard` já usa `guard_fail` para falha de verificação de assinatura em [`signing.rs:81`](../../../crates/dare-guard/src/signing.rs#L81) — mesma natureza de erro |
| `InvalidInput` (4) | Rejeitada. A entrada do usuário está correta; é a **evidência** que falhou |
| `Internal` (1) | Rejeitada. Não é bug do CLI; é o sistema funcionando como projetado |
| Novo `ErrorKind` | Rejeitada. Mudança na tabela pública de exit codes = breaking change por [`breaking-change-process.md`](../../../docs/compatibility/breaking-change-process.md) |

Nenhum código novo é criado: [`error.rs:31`](../../../crates/dare-core/src/error.rs#L31) já mapeia `GuardFail → 6`, com teste de estabilidade (`exit_code_mapping_is_stable`).

**Ressalva de compatibilidade, a registrar como Classe B.** Hoje `dare execute --complete` devolve exit **1** quando um gate Ralph reprova (`CoreError::internal` em [`execute.rs:301`](../../../crates/dare-cli/src/commands/execute.rs#L301)). O exit 6 é, portanto, um código novo **para esse comando**, ainda que não para a tabela global. Como só dispara com `handoff.strict = true`, que é opt-in e default `false`, nenhum usuário que não optou pode observá-lo — O-04 preservado. Isso entra explicitamente no documento de compatibilidade do ciclo e na matriz de classificação, não fica implícito.

### RF-09 — Proveniência mínima de workspace (MUST)

`startingContext` e `sourceRevision` podem apontar para revisões Git de **um ou mais** repositórios efetivamente usados:

- `repositoryUri` ou apelido, `commitSha`, e estado `clean` \| `dirty` \| `unattested`.
- Working tree modificada → `dirty`, com digest de working tree quando disponível. **Nunca** tratada como `clean`.
- Offline / sem git → `unattested` = evidência insuficiente para política estrita, nunca revisão fabricada.
- **Não** cria monorepo, não gerencia submódulos, não clona nada. Apenas registra e verifica o que já está presente.

Requer novas primitivas (HEAD SHA, dirty check) sobre `find_git_root`, via `SafeCommand` com timeout e limite de stdout — o padrão já usado em [`git.rs:33`](../../../crates/dare-project/src/git.rs#L33). Git ausente degrada para `None`, nunca erro.

### RF-10 — Isolamento do formato compartilhado (MUST)

- Caminho canônico próprio (D-02), separado de `.dare/state.json` e dos campos de `dare-dag.yaml`.
- **Nenhum** campo novo obrigatório em `.dare/state.json`. **Nenhuma** ampliação do `dare.config.json` gerado pelo `init` (a chave `handoff` é lida de `extra` quando presente; sua ausência significa `strict: false`).
- Teste de round-trip **Rust → npm → Rust**: o sidecar sobrevive a um `saveState` do npm, e o `state.json` reescrito pelo npm continua consumível pelo Rust.
- Executor npm que **não** reconhece recibos: cenário de fixture explícito. O npm compõe contexto como hoje (sem verificação) — é exatamente o comportamento de compatibilidade, documentado como tal. Em estrito, o Rust recusa contexto não verificável ainda que o npm o tenha aceitado.

### Diferidos para o PR 02

| RF | Título | Por quê |
|---|---|---|
| RF-06 | Consulta e diagnóstico (`SHOULD`) | Depende do formato congelado |
| RF-07 | Encadeamento declarado de dependências (`SHOULD`) | RF-05 já valida cada pai; falta só a declaração e a semântica configurável |
| RF-08 | Proteção contra reuso indevido (`SHOULD`) | RF-03 já marca `superseded`; falta a política completa de expiração |
| RF-11 | Revisão independente + Human Gate (`SHOULD`) | A metade "Human Gate" já é coberta por `--reset` (RF-02); a metade "autor ≠ verificador" exige modelo de identidade de runner |

---

## 5. Componentes existentes a reutilizar

| Componente | Papel neste ciclo |
|---|---|
| [`dare-contracts/src/dag.rs`](../../../crates/dare-contracts/src/dag.rs) | `DagTask`, `DagV21`, `LegacyDag` — ponto de ancoragem do contrato, sem reescrita do parser |
| [`dare-contracts/src/state.rs`](../../../crates/dare-contracts/src/state.rs) | `TaskRuntimeState`, `AttemptRecord` — **lidos**, não estendidos |
| [`dare-contracts/src/io.rs`](../../../crates/dare-contracts/src/io.rs) | `read_limited` (2 MiB), `write_json_atomic` — limites e escrita canônica de graça |
| [`dare-dag/src/execution.rs`](../../../crates/dare-dag/src/execution.rs) | `compose_task_prompt`, `parent_context_limit` — único ponto de consumo a alterar |
| [`dare-verify/src/verification.rs`](../../../crates/dare-verify/src/verification.rs) | `VerificationReport`, `write_verification`, `load_verification`, `task_id_is_path_safe` |
| [`dare-cli/src/commands/execute.rs`](../../../crates/dare-cli/src/commands/execute.rs) | `run_complete_inner` — onde o recibo é emitido |
| [`dare-core/src/fs/`](../../../crates/dare-core/src/fs/) | `atomic_write`, `FileLock`, `SafeRelativePath`, `ProjectRoot::resolve` |
| [`dare-core/src/lib.rs:35`](../../../crates/dare-core/src/lib.rs#L35) | `to_canonical_json_string` — base do digest determinístico |
| [`dare-core/src/redact.rs`](../../../crates/dare-core/src/redact.rs) | `redact` — aplicado a todo texto que entre em recibo/handoff |
| [`dare-parity`](../../../crates/dare-parity/) + `tests/{golden,security}` | Harness de paridade, goldens e proptest de paths/parsers |

**Não reimplementar:** `dare init`, parser de DAG, Ralph Loop, mecanismo de hash básico, monorepo/submódulos.

`dare-graph` modela `VerifiedBy` / `EvidencedBy` / `ProvenBy` ([`types.rs:80`](../../../crates/dare-graph/src/types.rs#L80)). São arestas do **grafo de conhecimento**, não do DAG de execução — este ciclo não as confunde nem as popula automaticamente.

---

## 6. Requisitos não funcionais

### Performance
- Verificação no consumidor: custo dominado por I/O de digest dos artefatos referenciados. Orçamento: **< 50 ms** por pai para contratos com até 32 `artifactRefs`, sem regressão mensurável no gate `PERF_REGRESSION_MAX=0.15` ([DEC-055](../../../docs/DECISION-LOG.md)).
- Digest sobre bytes em streaming, não carregando artefato inteiro em memória quando evitável.

### Determinismo
- Serialização canônica (chaves ordenadas) para contrato e recibo — digest reprodutível entre SOs.
- Paths normalizados com `/` na lógica; conversão nativa só na camada de I/O (política de disco, `disk-and-json-policy.md`).
- `FixedClock` nos goldens; nenhum timestamp de wall-clock dentro do payload que entra no digest de contrato.

### Confiabilidade
- Escrita atômica + `FileLock` para o sidecar, mesmo padrão de `.dare/state.json` (fail-fast em contenção, sem fila).
- Limites de tamanho reaproveitando `MAX_CONTRACT_BYTES` (2 MiB) — arquivo malicioso ou OOM barrado antes do parse.
- Falha de leitura do sidecar nunca corrompe `state.json`: os dois são independentes.

### Compatibilidade
- `handoff.strict` ausente ou `false` → saída observável idêntica à `main`.
- Round-trip npm → Rust preservado e testado.
- Nenhuma mudança na **tabela** pública de exit codes (seria breaking change por [`breaking-change-process.md`](../../../docs/compatibility/breaking-change-process.md)). O exit 6 passa a ser alcançável por `dare execute` apenas sob `handoff.strict = true` — Classe B documentada em RF-05, inobservável no default.

---

## 7. Segurança

Local-first: nenhuma chamada de rede, token ou serviço externo é introduzida.

| OWASP | Risco neste ciclo | Controle |
|---|---|---|
| **A01** Broken Access Control | Recibo referenciando artefato fora do projeto, ou de outra task | Toda referência via `SafeRelativePath` + `ProjectRoot::resolve`; recusa de path absoluto, `..`, symlink traversal e referência cruzada indevida → `HANDOFF_REF_ESCAPE` |
| **A02** Cryptographic Failures | Alegar autenticidade a partir de hash | D-04: SHA-256 = integridade, não assinatura. Nenhuma alegação de autoria no v1 |
| **A03** Injection | **Prompt injection** via contexto de upstream: hoje o `output` de um pai entra no prompt sem validação | Validar antes de compor (RF-05); handoff carrega resumo limitado + referências verificáveis, nunca log completo; `deliverable` passa por `redact` |
| **A04** Insecure Design | Prova circular — recibo emitido a partir de booleano manipulável | RF-03: o recibo é **derivado** do `VerificationReport` lido do disco e dos digests dos artefatos. Gate reprovado não emite recibo |
| **A05** Security Misconfiguration | Usuário acreditar que está em estrito quando não está, ou ligar o estrito num projeto sem contratos e achar que está protegido | Default `false` documentado; aviso de compatibilidade nunca afirma validade; o estado aparece em `data` do `--json`; preflight D-06.1 falha uma vez com `HANDOFF_STRICT_NO_CONTRACTS`, distinto da lacuna real por task |
| **A06** Vulnerable Components | Nova dependência de crypto/git | Preferir `sha2` (já no workspace) e `SafeCommand` sobre `git`; `cargo audit` + `cargo deny check` no gate |
| **A08** Software & Data Integrity Failures | **Núcleo deste ciclo** — importar resultado adulterado | Digest de contrato, artefato e verificação revalidados *imediatamente antes* do consumo (TOCTOU); 1 byte alterado invalida |
| **A09** Logging & Monitoring Failures | Recusa silenciosa ou sem causa | Motivo estruturado + código estável em toda recusa, humano e JSON |
| **A10** SSRF | — | Não aplicável: nada de rede |

Controles adicionais:

- **Segredos:** `redact` aplicado a todo texto que entre em recibo, handoff, prompt ou log — mesmo tratamento que `write_verification` já dá a `stdout_tail` / `stderr_tail`. Nenhum `.env*`, token ou PII em sidecar.
- **Aprovação humana:** mudança material do contrato após o início da task é **recusada**, com `dare execute --reset <task-id>` como caminho sancionado (RF-02). O reset é uma ação humana deliberada e, neste ciclo, cumpre o papel de Human Gate; o gate formal com autor ≠ verificador é RF-11 / PR 02. Nenhum merge ou push automático.
- **Supply chain:** nenhuma nova dependência de rede, nenhum download em tempo de execução.

---

## 8. Fora de escopo

- Motor novo de DAG, scheduler distribuído, orquestrador remoto, execução multi-repo.
- Assinatura remota, PKI, attestations públicas, validação de identidade de terceiros.
- Execução paralela em larga escala, auto-replanejamento, manutenção de Skills.
- Migração compulsória de contratos legados no primeiro release.
- Alterar o `dare init` / `dare bootstrap` ou os 27 goldens de paridade — só reexecutá-los como gate.
- Mudar o npm. Em compatibilidade, o executor npm segue compondo contexto sem verificação, e isso é documentado como comportamento esperado.
- RF-06, RF-07, RF-08, RF-11 (D-01).

---

## 9. Aceite

Os 8 casos obrigatórios da IDEIA §7, mais 3 derivados das decisões desta fase, cada um com fixture positiva e negativa:

| # | Cenário | Esperado |
|---|---|---|
| 1 | **Positivo** — contrato imutável + gates reais aprovados + digests corretos | Handoff aceito; prompt de B contém o deliverable e as referências |
| 2 | **Sem evidência** — `DONE` sem recibo | Estrito: bloqueio com `HANDOFF_RECEIPT_MISSING`. Compat: aviso identificável, prompt inalterado |
| 3 | **Adulteração** — alterar output, artefato ou contrato após aprovação | Importação negada com o código específico de cada caso, mesmo com `status == DONE` |
| 4 | **Gate reprovado** | Nenhum recibo `verified-deliverable` emitido; Ralph Loop não contornado |
| 5 | **Reexecução** | Evidência anterior preservada e marcada `superseded`; importação aponta para a revisão correta |
| 6 | **Pais múltiplos** | Cada pai validado independentemente; só os válidos são importados |
| 7 | **Segurança** | Symlink traversal, path absoluto externo, referência cruzada e segredo em relatório todos rejeitados |
| 8 | **Compatibilidade** | Suíte existente verde; 27 goldens de `init` npm inalterados; round-trip Rust → npm → Rust preserva o sidecar; documentação de migração aprovada |
| 9 | **Preflight (D-06.1)** — estrito ativo sobre DAG legacy e sobre v2.1, ambos sem nenhum contrato | Falha **uma vez**, antes de compor, com `HANDOFF_STRICT_NO_CONTRACTS` + exit 6. Com ao menos 1 contrato presente, o preflight passa e a lacuna por task cai em `HANDOFF_CONTRACT_MISSING` |
| 10 | **Recuperação por reset (RF-02)** — mudança material do contrato com a task em `RUNNING` | Conclusão recusada nomeando `--reset`; após `dare execute --reset <id>` + novo `Start`, o `contractDigest` é o do contrato corrigido, `attempts` preservados e o recibo anterior marcado `superseded` |
| 11 | **Exit code em estrito** | `handoff.strict = false` → exit e stdout byte-idênticos à `main` nos goldens de `dare execute`. `handoff.strict = true` com recusa → exit **6**, envelope `--json` com `error.kind = "GuardFail"` e o código em `data` |

Gates do Ralph Loop, com evidência real registrada em `PROOF.md`:

```
cargo fmt --all                                        # antes de commitar
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
cargo deny check
```

Falha em qualquer gate impede marcar DONE. CVE HIGH/CRITICAL em dependência = task FAILED até corrigir.

**Nenhum merge na `main` sem aprovação humana explícita.** PR entregue em Draft/Ready for review.

---

## 10. Riscos e trade-offs

| Risco | Mitigação |
|---|---|
| Quebra de compatibilidade | Estrito é opt-in, default `false` (D-03); O-04 medido por goldens |
| Prova circular | Recibo derivado da evidência em disco, nunca de booleano (RF-03) |
| Falsa confiança em Git | Working tree suja → `dirty`, nunca `clean`; sem git → `unattested` (RF-09) |
| Volume de contexto no prompt | Handoff carrega resumo limitado + referências, nunca log completo (RF-04) |
| Race / TOCTOU | Revalidação de digest e confinamento imediatamente antes do consumo (RF-05) |
| npm descartar extensão | Sidecar fora de `state.json` + teste de round-trip (RF-10 / D-02) |
| Custo de I/O na composição | Orçamento de performance em §6, medido contra o baseline 054 |

---

## 11. Próximas etapas

1. **Revisar e aprovar este DESIGN** (fase R — aprovação humana obrigatória).
2. `/dare-blueprint DARE/cycles/rust-verified-task-contracts-012/DESIGN.md` — decidir: crate de destino (novo `dare-handoff` vs. extensão de `dare-verify`), forma final do contrato no YAML, nomes finais dos códigos de recusa, onde mora o preflight D-06.1 (`dare-dag` junto da composição vs. camada CLI, como o `run_preflight` do guard), assinaturas de API.
3. `/dare-tasks` — decompor por contratos, storage, integração, compatibilidade, segurança, cenários positivos/negativos e documentação; gerar `dare-dag.yaml` e `EXECUTION/task-*.md`.
4. `dare validate` no DAG; então `/dare-execute` task a task com Ralph Loop completo.
5. `REGRESSION.md` + `PROOF.md` com evidência real; `docs/compatibility/` novo documento; `DEC-059` no `docs/DECISION-LOG.md`.
6. PR em Draft — **sem merge sem autorização humana específica**.
