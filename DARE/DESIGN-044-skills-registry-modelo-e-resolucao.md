# DESIGN: Skills registry — modelo e resolução (Microplano 044)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (ciclo autorizado sem pausa humana)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/044-skills-registry-modelo-e-resolucao.md`  
> **Referência:** Documento Mestre §4.1 · ADR-007 (skill-pacote ≠ capability IDE) · contratos 007 (`.dare/skills.yml`) · path safety 005 · assets 009  
> **Posição:** 44 de 56  
> **Arquivo:** `DARE/DESIGN-044-skills-registry-modelo-e-resolucao.md`  
> **Escopo deste ciclo:** modelo `RegistrySkill` / `SkillManifest`, registries mock/local/remoto, resolução topológica, CLI `dare skill list|info`. **Não** install/remove/update/publish (→ **045**).

---

## 1. DESCRIÇÃO

Portar a camada de **skills-pacote** (não capabilities de IDE): tipos de domínio, três fontes de registry com prioridade `remote > local > mock`, leitura do manifest legado `.dare/skills.yml`, e resolução de dependências com detecção de ciclos. Superfície CLI mínima: `dare skill list` e `dare skill info <nome>`.

Consumidores: engenheiros DARE, CI de paridade, e microplano **045** (lifecycle). Entrega: crate **`dare-skills`** + comando CLI + docs + DEC.

---

## 2. OBJETIVOS E MÉTRICAS

| # | Objetivo | Métrica | Meta |
|---|----------|---------|------|
| O-01 | Tipos `RegistrySkill` + `SkillManifest` | API pública tipada + serde | Unit |
| O-02 | Ler `.dare/skills.yml` | Reusar `dare-contracts::SkillsManifest` | Unit + fixture |
| O-03 | Registry mock | 7 skills embarcadas; lista determinística | Unit |
| O-04 | Registry local | `~/.dare/registry/**` ou `DARE_LOCAL_REGISTRY`; path-safe | Unit FS |
| O-05 | Registry remoto | Timeout 3 s; falha → fallback (não derruba comando) | Unit + mock HTTP |
| O-06 | Prioridade list/info | remote > local > mock (mesmo nome → vence o de maior prioridade) | Unit |
| O-07 | Resolve topológico | Ordem estável; ciclo → InvalidInput | Unit |
| O-08 | 6 genéricas vs stack | `SkillKind::{Generic,Stack}`; 6 IDs canônicos | Unit |
| O-09 | CLI `skill list\|info` | Help + human + `--json`; info NotFound=3 | Smoke |
| O-10 | Sem lockfile | DEC documenta ausência (paridade Mestre) | DEC |
| O-11 | Ralph | fmt + clippy `-D warnings` + test workspace | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Skills-pacote portáveis no rewrite Rust |
| Tech Lead | Sem lifecycle 045; sem ciclo de crates |
| Compat | Baseline TS 3.18.1; diffs classificados (DEC-033) |
| Segurança | Path jail; sem secrets em logs; timeout remoto |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Pri | Aceite |
|----|-----------|-----|--------|
| RF-01 | Crate `dare-skills` member do workspace | MUST | Sem ciclo com `dare-cli` |
| RF-02 | `RegistrySkill` | MUST | Campos: name, version, description, author, license, dare_version?, depends_on, kind, source |
| RF-03 | `SkillManifest` | MUST | Schema de `skill.yml` de pacote (name, version, description, author, license, dare_version?, depends_on?) |
| RF-04 | Ler `.dare/skills.yml` | MUST | Via `dare-contracts`; ausente → lista vazia / aviso em list (não exit 3) |
| RF-05 | Mock registry | MUST | `include_str!` JSON com **7** skills; única fonte offline garantida |
| RF-06 | Local registry | MUST | Root `~/.dare/registry` ou env `DARE_LOCAL_REGISTRY`; layout `<name>/<version>/` + `index.json` opcional |
| RF-07 | Remote registry | MUST | Base `https://dare-registry.vercel.app`; `GET /api/skills` e `GET /api/skills/<name>`; timeout **3 s**; erros soft |
| RF-08 | Prioridade | MUST | Merge list/info: remote > local > mock |
| RF-09 | `resolve_dependencies` | MUST | Kahn/topo; ciclo → `InvalidInput` com mensagem `dependency cycle` |
| RF-10 | Seis genéricas | MUST | `dare-ax`, `dare-frontend-design`, `dare-layered-design`, `dare-llm-integration`, `dare-quality-telemetry`, `dare-realtime` → `SkillKind::Generic` |
| RF-11 | Skill de stack | MUST | Nome `skill-*` ou não na lista genérica → `SkillKind::Stack` |
| RF-12 | CLI `dare skill list` | MUST | Lista merged ordenada por name (byte-wise); `--json` envelope |
| RF-13 | CLI `dare skill info <name>` | MUST | Detalhe merged; NotFound exit **3** |
| RF-14 | Sem add/remove/update/publish | MUST | Ausentes do clap |
| RF-15 | Lockfile | MUST | **Não** implementar; DEC-033 |
| RF-16 | Path safety | MUST | Nomes de skill/version sem `..` / absolutos; local sob root do registry |
| RF-17 | Mensagens en-US | MUST | Erros de domínio em inglês |
| RF-18 | Docs | MUST | `docs/compatibility/cli-skill.md` + DEC-033 |
| RF-19 | Matriz status 044 | MUST | `✅ Concluído` ao fechar |
| RF-20 | Hotspot `main.rs` | MUST | Só subcomando `Skill` + match arm mínimo |

### Superfície CLI

```text
dare skill list [--json]
dare skill info <name> [--json]
```

### Contratos de disco

| Path | Papel | Mutação 044 |
|------|-------|-------------|
| `.dare/skills.yml` | Manifest do projeto | Read-only |
| `~/.dare/registry/**` | Registry local | Read-only |
| `packages/skills/**` | Install target | Fora (045) |

---

## 5. REQUISITOS NÃO FUNCIONAIS

| ID | Requisito |
|----|-----------|
| RNF-01 | Timeout remoto ≤ 3 s; sem hang |
| RNF-02 | List determinístico (ordenação estável) |
| RNF-03 | Sem `unwrap()` em produção |
| RNF-04 | Cross-platform (home dir Windows/Unix) |

---

## 6. SEGURANÇA

| ID | Controle |
|----|----------|
| RS-01 | Validar name/version path-safe |
| RS-02 | Remoto: sem Bearer neste ciclo; não logar tokens |
| RS-03 | Soft-fail remoto (nunca Internal por timeout de rede) |
| RS-04 | Redação via `CoreError` factories |

---

## 7. FORA DE ESCOPO

- `dare skill add|remove|update|publish` (045)
- Assinatura minisign de skills (guard/045+)
- Capabilities IDE / harness adapters
- Lockfile / integrity hash
- Alterar schema de `dare-contracts::SkillsManifest` sem ADR

---

## 8. DECISÕES ABERTAS → BLUEPRINT

| # | Tema | Default Design |
|---|------|----------------|
| D-01 | HTTP client | `ureq` com timeout 3 s |
| D-02 | Soft-fail remoto | Tratar como fonte vazia + tracing warn |
| D-03 | Info vs TS (mock-only) | Classe B: Rust usa prioridade full |
| D-04 | 7ª skill mock | `skill-nestjs-api` (stack) |
| D-05 | DEC | DEC-033 |

---

## 9. CRITÉRIOS DE ACEITE (resumo)

- [ ] list/info prioridade remote > local > mock
- [ ] ciclos detectados
- [ ] fallback remoto não derruba comando
- [ ] fmt / clippy `-D warnings` / test workspace OK
- [ ] matriz 044 → Concluído
- [ ] sem install/publish no clap
