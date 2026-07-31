# BLUEPRINT: Skills registry — modelo e resolução (Microplano 044)

> **Gerado a partir de:** `DARE/DESIGN-044-skills-registry-modelo-e-resolucao.md` v1.0  
> **Data:** 2026-07-22 | **Status:** APPROVED (ciclo autorizado)  
> **Arquivo:** `DARE/BLUEPRINT-044-skills-registry-modelo-e-resolucao.md`  
> **Pré-requisitos:** 005 path · 007 contracts · 009 assets  
> **Escopo:** `dare-skills` + `dare skill list|info`. **Não** lifecycle 045.

---

## 0. TRADE-OFFS

| # | Trade-off | Escolha | Justificativa |
|---|-----------|---------|---------------|
| T-01 | Crate | `crates/dare-skills` | Microplano |
| T-02 | Deps | `dare-core`, `dare-contracts`, serde, serde_json, serde_yaml, ureq | Sem ciclo CLI |
| T-03 | HTTP | `ureq` 3 s timeout | Leve; timeout nativo |
| T-04 | Soft-fail remote | Erro → `Ok(None)` / lista parcial; warn tracing | RF-07; aceite |
| T-05 | Prioridade | remote > local > mock | Aceite microplano (Classe B vs TS info=mock-only) |
| T-06 | Project manifest | Reusar `dare-contracts::load_skills_manifest` | RF-04 |
| T-07 | Package schema | `SkillManifest` em `model.rs` (`skill.yml`) | Distinto de `SkillsManifest` |
| T-08 | Mock | `data/registry-mock.json` via `include_str!` | 7 skills |
| T-09 | Local root | `dirs::home_dir()/.dare/registry` ou `DARE_LOCAL_REGISTRY` | Mestre |
| T-10 | Lockfile | **Ausente** | DEC-033; Mestre §4.1 |
| T-11 | Topo | Kahn; ordem estável por name após indegree | Ciclo → InvalidInput |
| T-12 | Kind | `GENERIC_SKILL_IDS` const array (6); else Stack se `skill-` prefix ou unknown | RF-10/11 |
| T-13 | CLI | `commands/skill.rs` + `Commands::Skill` | Minimizar main.rs |
| T-14 | Docs | `cli-skill.md` + DEC-033 | RF-18 |
| T-15 | Exit | 0 ok; 2 usage; 3 not found (info); 4 invalid; 5 io | Padrão core |

### 0.1 Constantes

| Nome | Valor |
|------|-------|
| `REMOTE_BASE_URL` | `https://dare-registry.vercel.app` |
| `REMOTE_TIMEOUT` | 3 s |
| `ENV_LOCAL_REGISTRY` | `DARE_LOCAL_REGISTRY` |
| `ENV_REMOTE_URL` | `DARE_REMOTE_REGISTRY` (override testes) |
| `PROJECT_SKILLS_REL` | `.dare/skills.yml` |
| `GENERIC_SKILL_IDS` | 6 nomes canônicos |

### 0.2 Exit codes

| Code | Quando |
|------|--------|
| 0 | list/info OK |
| 2 | Usage (subcommand ausente) |
| 3 | `info` skill não encontrada em nenhuma fonte |
| 4 | InvalidInput (ciclo, name path-unsafe, config) |
| 5 | Io (leitura local corrompida de forma hard — preferir skip entry) |

---

## 1. ARQUITETURA

```text
dare-cli (skill list|info)
    └── dare-skills
            ├── model.rs      RegistrySkill, SkillManifest, SkillKind, SkillSource
            ├── registry.rs   Mock / Local / Remote / Composite + resolve
            └── data/registry-mock.json
    └── dare-contracts        SkillsManifest (.dare/skills.yml)
    └── dare-core             CoreError, path helpers
```

### 1.1 Tipos (congelados)

```rust
pub enum SkillKind { Generic, Stack }
pub enum SkillSource { Mock, Local, Remote }

pub struct RegistrySkill {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub dare_version: Option<String>,
    pub depends_on: Vec<String>,
    pub kind: SkillKind,
    pub source: SkillSource,
}

pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub dare_version: Option<String>,
    pub depends_on: Vec<String>,
}
```

### 1.2 APIs públicas

```rust
pub fn classify_skill(name: &str) -> SkillKind;
pub fn validate_skill_id(name: &str) -> CoreResult<()>;
pub fn load_project_skills(root: &ProjectRoot) -> CoreResult<SkillsManifest>; // empty if missing
pub fn resolve_dependencies(skills: &[RegistrySkill], roots: &[String]) -> CoreResult<Vec<String>>;
pub struct CompositeRegistry { /* mock+local+remote */ }
impl CompositeRegistry {
    pub fn from_env() -> Self;
    pub fn list(&self) -> CoreResult<Vec<RegistrySkill>>;
    pub fn get(&self, name: &str) -> CoreResult<Option<RegistrySkill>>;
}
```

### 1.3 Merge

Para cada `name`, escolher a entrada da fonte de maior prioridade presente. Ordenar resultado final por `name` ASCII.

### 1.4 Local layout

```text
<registry-root>/
  index.json                 # opcional: [{ "name", "version", ... }]
  <name>/<version>/
    skill.yml                # SkillManifest
```

Se `index.json` ausente: scan de diretórios path-safe.

### 1.5 Remote soft-fail

Qualquer erro de rede/parse/timeout → tracing::warn + tratar fonte como vazia (list) ou None (get). Comando continua com local+mock.

---

## 2. TASKS (DAG)

| ID | Título | depends_on |
|----|--------|------------|
| mp044-001 | Scaffold `dare-skills` + workspace wire | [] |
| mp044-002 | `model.rs` + mock JSON + classify | [mp044-001] |
| mp044-003 | Registries local/remoto/composite + resolve | [mp044-002] |
| mp044-004 | CLI `skill list\|info` + smokes | [mp044-003] |
| mp044-005 | Docs compat + DEC-033 + matriz status | [mp044-004] |

---

## 3. TESTES

| Caso | Esperado |
|------|----------|
| classify 6 genéricas | Generic |
| classify `skill-nestjs-api` | Stack |
| mock list len=7 | ordenado |
| merge remote wins | source=Remote |
| remote timeout | list ainda OK com mock |
| cycle A→B→A | InvalidInput cycle |
| topo dare-ax first | ordem deps |
| info missing | exit 3 |
| path `../evil` name | InvalidInput |

---

## 4. DEC / COMPAT

| Item | Classe | Nota |
|------|--------|------|
| Prioridade info full vs TS mock-only | B | DEC-033 |
| Sem lockfile | A (preservar) | Mestre |
| Soft-fail remote | A | Paridade timeout never throws |

---

## 5. GAP

| Item | Estado |
|------|--------|
| `dare-skills` | 🔴 criar |
| CLI skill | 🔴 criar |
| lifecycle | ⬜ 045 |
