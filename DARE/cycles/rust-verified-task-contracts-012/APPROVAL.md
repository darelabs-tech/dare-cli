# APPROVAL

Registro das aprovações humanas e das decisões tomadas quando a resposta não foi explícita.

## Aprovações

- **2026-10-13** — DESIGN revisado e aprovado ("design aprovado pode seguir com o blueprint").
- **2026-10-13** — BLUEPRINT revisado e aprovado ("aprovado pode seguir").

## Decisões do usuário na fase de Design

Respondidas explicitamente, as três pendências que a IDEIA §13 deixou para o DESIGN:

| Decisão | Escolha do usuário |
|---|---|
| **D-01** Escopo do ciclo | Somente os MUST — RF-01..05, 09, 10. RF-06, RF-07, RF-08 e RF-11 ficam para o PR 02 |
| **D-02** Storage | Contrato em `DARE/cycles/<ciclo>/contracts/` (versionado no Git) + evidência em `.dare/handoff/` (sidecar privado) |
| **D-03** Modo estrito | `handoff.strict` em `dare.config.json` com default `false`, mais `--strict-handoff` / `--no-strict-handoff` |

## Pontos sem resposta explícita; seguido o padrão recomendado

Levantados na revisão e mantidos conforme a recomendação, sem objeção do usuário:

- **D-06 / D-06.1 — DAG legacy em modo estrito.** Contrato é opcional; o estrito recusa em vez de inventar contrato vazio. Acrescentado o preflight de ativação, que distingue misconfiguração (`HANDOFF_STRICT_NO_CONTRACTS`, uma falha no início) de lacuna real por task (`HANDOFF_CONTRACT_MISSING`, falha no consumo).
- **RF-11 fora do escopo.** Mudança material de contrato é **recusada**, não aprovada por gate. O caminho de recuperação sancionado é `dare execute --reset <task>` — uma ação humana deliberada que cumpre o papel de Human Gate neste ciclo. A metade "autor ≠ verificador" exige modelo de identidade de runner e fica no PR 02.
- **Envelope `--json` intocado.** Códigos de recusa vão em `data`, não em `error.kind`. A recusa estrita mapeia para `CoreError::GuardFail` → exit **6**, código já existente na tabela pública. Registrado como **Classe B**: o exit 6 é novo para `dare execute`, mas inalcançável com o default `strict = false`.

## Decisões técnicas tomadas nas fases A e E

Derivadas da leitura do código real, sem alterar o escopo aprovado:

- **A-01 / A-02** — Nenhum crate novo. `dare-dag` não depende de `dare-verify`; a ligação é por inversão de dependência (`trait ParentEvidence`).
- **A-04** — Diretório de contratos = irmão `contracts/` do DAG resolvido.
- **A-05** — **Dois** digests de contrato (`contractDigest` material e `contractFullDigest`), corrigindo uma contradição do DESIGN em que um digest único não podia simultaneamente cobrir o contrato inteiro e deixar mudança não material passar.
- **A-06** — Estratégia de artefatos em tri-state de revisão (`clean` / `dirty` / `unattested`), porque "digest dos bytes efetivos" não dizia **quais** artefatos e a expansão completa dos globs seria ampla demais.
- **A-08** — Zero dependência nova.
- **§7** — `dare-verify` ganha dois arquivos (`handoff.rs` e `handoff_store.rs`) para que `task-005` e `task-006`, no mesmo rank, não editem o mesmo arquivo.

## Limites da aprovação

- **A aprovação não cobre merge na `main`.** O PR fica em Draft/Ready for review, sem merge sem autorização humana específica.
- A aprovação não cobre ativar `handoff.strict` por default em nenhum projeto.
- Os artefatos deste ciclo ficam sob `DARE/`, que está no `.gitignore` (linha 21, repo público). Como nos ciclos anteriores, commitá-los exige `git add -f` — decisão do usuário.
