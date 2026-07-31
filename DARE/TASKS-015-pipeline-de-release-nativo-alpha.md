# Tasks: Pipeline de release nativo alpha (015)

> **Fonte:** `DARE/BLUEPRINT-015-pipeline-de-release-nativo-alpha.md`  
> **Design:** `DARE/DESIGN-015-pipeline-de-release-nativo-alpha.md`  
> **DAG:** `DARE/dare-dag-015.yaml`  
> **Specs:** `DARE/EXECUTION-015/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** DEC-016 / ADR-008 fechados; próximo: `016-comando-welcome`

## Visão Geral

- Total de Tasks: 7
- Status: **DONE** — microplano 015 fechado; próximo: `016-comando-welcome`

## Tabela de Status

| ID        | Título                                                      | Status  | Depends On           | Complexity |
|-----------|-------------------------------------------------------------|---------|----------------------|------------|
| mp015-001 | Verificar Dockerfile.rust + docker-compose.ci.yml           | ✅ DONE | —                    | LOW        |
| mp015-002 | Congelar release.yml matrix 5 + package + dry_run           | ✅ DONE | —                    | MED        |
| mp015-003 | SHA256SUMS + SBOM + cosign soft + publish                   | ✅ DONE | mp015-002            | MED        |
| mp015-004 | Installers install.sh + install.ps1                         | ✅ DONE | —                    | MED        |
| mp015-005 | Smoke instalação limpa + docs DEC-016 + ADR-008             | ✅ DONE | mp015-003, mp015-004 | MED        |
| mp015-006 | Auditoria Ralph (test/clippy/audit/deny)                    | ✅ DONE | mp015-001, mp015-005 | MED        |
| mp015-007 | Fechamento microplano 015                                   | ✅ DONE | mp015-006            | LOW        |

## Próximas Etapas

1. Microplano **016** — comando welcome (`016-comando-welcome`)
