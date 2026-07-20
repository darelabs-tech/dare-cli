# Inventário de fixtures de regressão (RF-12)

Catálogo SHOULD de fixtures canônicas do Ciclo 0 usadas em shadow tests, golden runner e verificação de paridade com a baseline npm `@dewtech/dare-cli@3.18.1`. Materialização física prevista nos microplanos de execução (055); este inventário fixa nomes e propósito.

| fixture_id | propósito |
|------------|-----------|
| `empty-project` | Diretório greenfield sem stack detectável — valida `discover`/`init` em projeto vazio |
| `existing-node-project` | Projeto Node.js brownfield — valida detecção de stack e paridade de comandos |
| `existing-rust-project` | Projeto Rust/Cargo existente — valida detecção de workspace e toolchain |
| `existing-python-project` | Projeto Python existente — valida detecção brownfield de stack Python |
| `monorepo` | Layout monorepo com múltiplos pacotes — valida resolução de paths e escopo |
| `project-with-claude` | Projeto com harness Claude Code (`.claude/`) — valida adapter de capabilities |
| `project-with-cursor` | Projeto com harness Cursor (`.cursor/`) — valida adapter de capabilities |
| `project-with-codex` | Projeto com harness Codex — valida adapter de capabilities |
| `project-with-antigravity` | Projeto com harness Antigravity — valida adapter de capabilities |
| `project-with-all-harnesses` | Projeto com todos os harnesses IDE — valida matriz completa de capabilities |
| `invalid-config` | `dare.config.json` malformado ou inválido — valida exit codes e mensagens de erro |
| `legacy-dag` | `DARE/dare-dag.yaml` legado — valida leitura e migração sem perda silenciosa |
| `customized-assets` | Assets DARE customizados pelo usuário — valida merge/preservação no `update` |
| `windows-path-cases` | Paths Windows (backslash, drive letters) — valida paridade cross-platform |

## Referências

- Baseline: [`baseline-manifest.json`](baseline-manifest.json), [`baseline-3.18.1.md`](baseline-3.18.1.md)
- Matriz de classificação: [`classification-matrix.md`](classification-matrix.md)
- Épico de rastreamento RF-01–RF-11: ver [`../DECISION-LOG.md`](../DECISION-LOG.md) (RF-13)
