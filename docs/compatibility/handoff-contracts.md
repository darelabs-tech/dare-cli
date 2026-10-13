# Task contracts e handoffs verificados (Ciclo `rust-verified-task-contracts-012`)

Contrato público do ciclo. Define o que fica em disco, o que o CLI recusa e o
que permanece inalterado para quem não optou pelo recurso.

**O recurso é opt-in e o default não mudou.** Sem `handoff.strict` e sem
contratos em disco, `dare execute` produz exatamente o stdout e os exit codes
que produzia antes deste ciclo — asserido byte a byte em
`crates/dare-cli/tests/handoff_compat.rs::strict_off_byte_identical` contra
snapshots gravados antes de qualquer código de handoff existir.

## Digest não é assinatura

Os digests deste ciclo são **SHA-256 de integridade**. Provam que uma sequência
de bytes não mudou; não dizem nada sobre **quem** a produziu. Não há
autenticação de autor, nem cadeia de confiança, nem verificação de origem.
Tratar um recibo como prova de autoria é leitura errada do artefato.

Assinatura real existe em outro lugar do produto (`cosign verify-blob` em
`dare self update`, ADR-008) e não se aplica aqui.

## Onde cada coisa mora

| Artefato | Caminho | Versionado em Git | Quem escreve |
|---|---|---|---|
| Task contract | `<dir do DAG>/contracts/<task-id>.yaml` | **Sim** — é intenção declarada, revisável em PR | humano |
| Handoff sidecar | `.dare/handoff/<task-id>.json` | Não — é evidência de execução local | `dare execute --complete` |
| Relatório de verificação | `.dare/verification/<task-id>.json` | Não | Ralph Loop |

A separação é deliberada. O contrato é o que se promete e pertence à revisão
humana; o sidecar é o que se provou e pertence à máquina que executou.

O sidecar é **arquivo próprio**, não um campo de `.dare/state.json`. O runtime
npm reconstrói o state file a partir dos campos que conhece e descarta o resto,
então um sidecar em arquivo separado sobrevive a um ciclo pelo npm por
construção (RF-10).

## Schema do contrato

| Campo | Tipo | Obrigatório | Nota |
|---|---|---|---|
| `schemaVersion` | `u32` | sim | `1` |
| `taskId` | `string` | sim | precisa existir no DAG e casar com o nome do arquivo |
| `goal` | `string` | sim | ≤ 500 chars |
| `scope.allowedPaths` | `string[]` | sim | globs relativos; `**` sozinho é advertência |
| `scope.deniedPaths` | `string[]` | não | avaliado **depois** de `allowedPaths` |
| `acceptance[].id` | `string` | sim | ≤ 64 critérios |
| `acceptance[].assertion` | `string` | sim | ≤ 500 chars |
| `verification.requiredGates` | `string[]` | sim | subconjunto de `build`, `test`, `lint` |
| `exitCriteria.kind` | enum | sim | `verified-deliverable` \| `follow-up-handoff` |
| `startingContext.source` | enum | sim | `project-revision` \| `parent-handoff` \| `none` |
| `startingContext.repositories[].alias` | `string` | não | ≤ 8 repositórios |
| `taskType`, `owner`, `requirementRefs` | opcionais | não | **fora** da projeção material |

Chaves desconhecidas são preservadas em `extra` e não invalidam o contrato.

### Dois digests, de propósito

| Digest | Cobre | Divergência significa |
|---|---|---|
| `contractDigest` | conjunto **fixo** de 8 chaves: `schemaVersion`, `taskId`, `goal`, `scope`, `acceptance`, `verification`, `exitCriteria`, `startingContext` | **recusa** — `HANDOFF_CONTRACT_DIGEST_MISMATCH` |
| `contractFullDigest` | contrato canônico inteiro, `extra` incluído | trilha auditável, **sem** recusa |

A projeção material é um conjunto fixo de chaves, não "tudo menos os
opcionais". Um campo novo adicionado ao tipo no futuro **não** entra na
projeção por acidente, então corrigir `owner` ou acrescentar um
`requirementRefs` nunca invalida uma importação já feita.

Um segredo colado em `owner` passa por `redact` antes de qualquer digest, o que
move `contractFullDigest` e deixa `contractDigest` intacto.

## Schema do sidecar

| Campo | Tipo | Nota |
|---|---|---|
| `schemaVersion` | `u32` | `1` |
| `taskId` | `string` | segmento de path seguro |
| `kind` | enum | `verified-deliverable` \| `follow-up-handoff` |
| `deliverable` | `string` | redigido na escrita |
| `contextBounds` | `u32` | limite de contexto do DAG |
| `nextSteps[]`, `openItems[]` | `string[]` | redigidos na escrita |
| `superseded[]` | objeto | histórico FIFO, no máximo **16** entradas |
| `receipt` | objeto | abaixo |

### `receipt`

| Campo | Nota |
|---|---|
| `contractDigest` / `contractFullDigest` | ver acima |
| `sourceRevision.state` | `clean` \| `dirty` \| `unattested` |
| `sourceRevision.repositories[].commitSha` | `null` quando não atestável — **nunca** um SHA fabricado |
| `artifactRefs[]` / `artifactDigests[]` | mesmo índice = mesmo arquivo; comprimentos iguais é invariante |
| `verificationRef` / `verificationDigest` | path e digest do relatório lido do disco |
| `verdict` | `passed` \| `failed` \| `insufficient` |
| `producer.cliVersion` / `producer.driver` | quem emitiu |

Um `follow-up-handoff` sem nenhum `openItems` é recusado na escrita: um
follow-up sem pendência declarada é um `DONE` disfarçado.

## Estratégia de artefatos (tri-state de revisão)

| `sourceRevision.state` | `artifactRefs` / `artifactDigests` | `worktreeDigest` |
|---|---|---|
| `clean` | arquivos alterados ∩ `allowedPaths`, ordenados, digest por arquivo | ausente |
| `dirty` | idem | digest da árvore sobre `allowedPaths` |
| `unattested` | vazio | digest da árvore sobre `allowedPaths` |

Com `unattested` o recibo **é** emitido. A insuficiência é decidida no
**consumo**, pela política, não na emissão — é o que permite rodar sem Git.

Symlinks **não** são seguidos: são contados em `skippedSymlinks` e excluídos do
digest. Seguir um symlink durante a expansão de globs seria o vetor de escape
que o confinamento fecha.

## Os 13 códigos de recusa

| Código | Significado |
|---|---|
| `HANDOFF_STRICT_NO_CONTRACTS` | modo estrito ligado e nenhum contrato encontrado — erro de configuração, falha **uma vez** no preflight |
| `HANDOFF_CONTRACT_MISSING` | a política exige contrato e esta task não tem |
| `HANDOFF_RECEIPT_MISSING` | não existe sidecar para o pai |
| `HANDOFF_RECEIPT_INVALID` | sidecar não desserializa, schema diferente de 1, ou `refs`/`digests` com comprimentos distintos |
| `HANDOFF_CONTRACT_DIGEST_MISMATCH` | o contrato em disco **agora** não bate com o digest material do recibo |
| `HANDOFF_ARTIFACT_DIGEST_MISMATCH` | um artefato mudou desde a emissão |
| `HANDOFF_VERIFICATION_DIGEST_MISMATCH` | o relatório de verificação mudou desde a emissão |
| `HANDOFF_GATE_NOT_SATISFIED` | um gate exigido pelo contrato não passou, provado **por aspecto** |
| `HANDOFF_VERDICT_NOT_PASSED` | o recibo registra `failed` ou `insufficient` |
| `HANDOFF_NOT_FINAL` | o handoff não é do tipo que o consumidor exige |
| `HANDOFF_REF_ESCAPE` | um path do recibo sai do projeto (absoluto, `..`, drive, UNC) |
| `HANDOFF_SIZE_LIMIT` | algum cap de leitura foi atingido |
| `HANDOFF_REVISION_UNATTESTED` | a política exige revisão atestada e esta não é |

Toda recusa carrega o código na mensagem, e o `detail` nomeia o path ofensor ou
um prefixo de **12 caracteres** do digest — diagnosticável sem vazar conteúdo.

### Ordem das checagens é contrato

As doze checagens de consumo rodam em ordem fixa e **a primeira falha ganha**.
Um pai quebrado de duas formas reporta a causa de número menor. Sem ordem fixa,
a mensagem de recusa seria não determinística e inútil para diagnóstico. Há
teste para isso.

Três checagens — contrato, relatório de verificação e artefatos — **releem o
disco no momento da chamada** e nunca reaproveitam valor lido antes no mesmo
processo. É o que faz "alterar um byte depois de verificar invalida a
importação" valer na janela entre verificar e consumir; sob `--agent`, a
revalidação acontece **por tentativa**.

## Política e defaults

Lido de `dare.config.json`, do objeto `handoff`. **`dare init` não escreve essa
chave**; ausência total significa todos os defaults.

| Chave | Default |
|---|---|
| `handoff.strict` | `false` |
| `handoff.requireContracts` | igual ao valor resolvido de `strict` |
| `handoff.requireAttestedRevision` | `false` |

Precedência: `--strict-handoff` > `--no-strict-handoff` > config > `false`.

O tipo exigido é sempre `verified-deliverable` nesta versão.

## Exit codes

| Situação | `strict` | Exit | `error.kind` no `--json` |
|---|---|---|---|
| Composição normal | off | `0` | — |
| Pai sem evidência | off | `0` | — (aviso em **stderr**) |
| Composição normal | on | `0` | — |
| Pai sem evidência válida | on | **`6`** | `GuardFail` |
| Estrito sem nenhum contrato | on | **`6`** | `GuardFail` |
| Contrato mudou entre start e complete | qualquer | `5` | `Config` |

**Exit 6 é inalcançável sem optar pelo recurso.** É Classe B na matriz de
classificação: novo para o comando, não para a tabela global de exit codes.

## Modo de compatibilidade

Com `strict` off e algum sidecar presente no projeto, uma recusa imprime em
**stderr** e o contexto é importado de todo jeito:

```
warning: handoff from task-002 not verified (HANDOFF_RECEIPT_MISSING); imported anyway
         (compatibility mode — this is not verification; enable handoff.strict to block)
```

**stdout não muda.** O aviso nunca afirma validade — diz explicitamente que não
é verificação.

Um projeto que nunca escreveu um sidecar não paga nada: nenhuma avaliação
acontece e stderr fica vazio.

## O CLI npm compõe contexto sem verificação

**Isso é comportamento de compatibilidade esperado, não bug.**

O executor npm (`@dewtech/dare-cli`) não conhece recibos. Ele compõe o contexto
do pai a partir de `.dare/state.json` e segue, exatamente como o lado Rust fazia
antes deste ciclo. Consequências:

- Uma task concluída pelo npm **não** produz sidecar. Um run Rust posterior em
  modo compat avisa e importa; em modo estrito, recusa.
- O `saveState` do npm descarta chaves desconhecidas, então o digest de start
  gravado em `extra` desaparece após um ciclo pelo npm. Ausência é lida como
  "nenhum digest registrado", **nunca** como divergência.
- O sidecar em si sobrevive intacto, porque é arquivo separado.

Coberto por `npm_roundtrip_preserves_sidecar` e `npm_executor_ignores_receipts`,
que simulam o writer do npm sem exigir Node no CI.

## Caps

| Leitura | Cap |
|---|---|
| Contrato | 2 MiB |
| Sidecar | 2 MiB |
| Arquivo único no digest | 64 MiB |
| Árvore no digest | 64 MiB |
| Arquivos referenciados | 512 |
| Histórico `superseded` | 16 (FIFO, descarta o mais antigo) |

## Fora de escopo nesta versão

RF-06, RF-07, RF-08 e RF-11 ficaram para o PR 02. Nada neste documento descreve
comportamento planejado — somente o que está implementado e testado.
