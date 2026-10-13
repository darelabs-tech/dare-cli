# TASKS — Paridade do `dare init` Rust com o npm 3.18.1

| ID | Task | Depende de | Complexidade | Status |
|---|---|---|---|---|
| task-001 | Golden do npm com shims de comandos | — | MED | PENDING |
| task-002 | Importar templates de stacks, IDE e método para `assets/` | — | MED | PENDING |
| task-003 | Modelo `ProjectConfig` e validação compatível com o npm | — | LOW | PENDING |
| task-004 | Renderer + scaffolders de backend/MCP (11 stacks, overlay Rails) | 002, 003 | HIGH | PENDING |
| task-005 | `StackTool` (auto/native/docker) + Vite, Rails, Leptos | 003 | HIGH | PENDING |
| task-006 | Layout por estrutura, workspace Rust, arquivos comuns, rollback | 004, 005 | HIGH | PENDING |
| task-007 | Instaladores por IDE e geradores de regras (B-01, B-02, B-03, B-05) | 002, 003 | HIGH | PENDING |
| task-008 | Prompts interativos e flags idênticos ao npm | 003 | MED | PENDING |
| task-009 | Ligar o pipeline ao `init`, saída de console, compat. do `bootstrap` | 006, 007, 008 | MED | PENDING |
| task-010 | Suíte de paridade contra o golden do npm | 001, 009 | HIGH | PENDING |
| task-011 | Docs, gates de qualidade e evidências de revisão | 010 | MED | PENDING |

Ranks do DAG:
- **0:** 001, 002, 003
- **1:** 004, 005, 007, 008
- **2:** 006
- **3:** 009
- **4:** 010
- **5:** 011

Nenhuma task é marcada DONE sem passar no Ralph Loop: build, test, lint e audit.
