# BLUEPRINT: Skills lifecycle e publish seguro (Microplano 045)

> **Gerado a partir de:** `DARE/DESIGN-045-skills-lifecycle-e-publish-seguro.md` v1.0  
> **Data:** 2026-07-24 | **Status:** APPROVED (ciclo autorizado)  
> **Arquivo:** `DARE/BLUEPRINT-045-skills-lifecycle-e-publish-seguro.md`  
> **Pré-requisitos:** 044 registry (`dare-skills` list/info) · 005 path · 007 contracts  
> **Escopo:** install/remove/update/publish. **Não** refine/patterns/graph.

---

## 0. TRADE-OFFS

| # | Trade-off | Escolha | Justificativa |
|---|-----------|---------|---------------|
| T-01 | Módulos | `install.rs` + `publish.rs` | Microplano |
| T-02 | CLI | Extender `SkillAction` additive | RF-10 Design |
| T-03 | Bugs TS | Corrigir (Classe C) | Mestre §35; DEC-043 |
| T-04 | Archives | `tar`+`flate2`+`zip` | Traversal block ambos |
| T-05 | Sign | Ed25519 in-crate (não dare-guard) | RNF-04 |
| T-06 | Manifest id | `SkillEntry.id` = skill name | Contracts 007 |
| T-07 | Staging | `packages/skills/.staging-<name>/` | Atomicidade |
| T-08 | Mock/remote files | Synthesize skill.yml + SKILL.md | Offline |
| T-09 | DEC | DEC-043 | User/scope |
| T-10 | Lockfile | Continua ausente | DEC-033 |

### 0.1 Constantes

| Nome | Valor |
|------|-------|
| `PACKAGES_SKILLS_REL` | `packages/skills` |
| `PROJECT_SKILLS_REL` | `.dare/skills.yml` |
| `ENV_SKILL_PRIVATE_KEY` | `DARE_SKILL_PRIVATE_KEY` |
| `SIG_EXT` | `.minisig` |
| `REQUIRED_LICENSE` | `MIT` |

### 0.2 Exit codes

| Code | Quando |
|------|--------|
| 0 | OK |
| 2 | Usage |
| 3 | Skill not found (registry / not installed) |
| 4 | InvalidInput (traversal, MIT, reverse-deps, unsafe name) |
| 5 | Io |

---

## 1. ARQUITETURA

```text
dare-cli (skill add|remove|update|publish|+list|info)
    └── dare-skills
            ├── model.rs / registry.rs   (044)
            ├── install.rs               NEW
            └── publish.rs               NEW
```

### 1.1 APIs públicas (`install.rs`)

```rust
pub fn install_skill(root: &ProjectRoot, name: &str, opts: &InstallOpts) -> CoreResult<InstallReport>;
pub fn remove_skill(root: &ProjectRoot, name: &str) -> CoreResult<RemoveReport>;
pub fn update_skill(root: &ProjectRoot, name: &str, opts: &InstallOpts) -> CoreResult<InstallReport>;
pub fn extract_archive_safe(archive: &Path, dest: &Path) -> CoreResult<()>; // tar.gz|tar|zip
```

### 1.2 APIs públicas (`publish.rs`)

```rust
pub fn validate_for_publish(manifest: &SkillManifest) -> CoreResult<()>;
pub fn pack_skill_dir(skill_dir: &Path, out_tar_gz: &Path) -> CoreResult<String>; // returns sha256 hex
pub fn publish_skill(root: &ProjectRoot, name: &str, out_dir: &Path) -> CoreResult<PublishReport>;
pub fn sign_artifact(path: &Path, private_key_hex: &str) -> CoreResult<()>;
```

### 1.3 Fluxo add

1. `validate_skill_id(name)`
2. Resolver skill no `CompositeRegistry` (ou `--from` archive)
3. `resolve_dependencies` → instalar cada dep ausente (staging)
4. Materializar conteúdo no staging
5. Rename atômico para `packages/skills/<name>`
6. Upsert `.dare/skills.yml`

### 1.4 Fluxo remove

1. Verificar instalado
2. Scan reverse deps em `packages/skills/*/skill.yml`
3. Se dependentes → InvalidInput
4. Remover dir + entrada manifest

### 1.5 Path safety archive

Rejeitar entry se: vazio, contém `\0`, `Component::ParentDir`, absoluto/`Prefix`, ou path normalizado sai de `dest`.

---

## 2. TASKS (DAG)

| ID | Título | depends_on |
|----|--------|------------|
| mp045-001 | DESIGN+BLUEPRINT+TASKS+dag+EXECUTION | [] |
| mp045-002 | `install.rs` atomic add/remove/update + traversal | [mp045-001] |
| mp045-003 | `publish.rs` pack/hash/sign + MIT/dare_version | [mp045-001] |
| mp045-004 | CLI SkillAction extend + smokes | [mp045-002, mp045-003] |
| mp045-005 | Docs cli-skill + DEC-043 + matriz | [mp045-004] |

---

## 3. TESTES

| Caso | Esperado |
|------|----------|
| add mock dare-ax | dir + skills.yml |
| remove apaga files | dir gone |
| remove com reverse dep | exit 4 |
| update recopia | conteúdo novo |
| tar `../evil` | InvalidInput |
| zip `../evil` | InvalidInput |
| publish sem dare_version | InvalidInput |
| publish OK | tar.gz + sha256 |
| help contém add/publish | smoke |

---

## 4. DEC / COMPAT

| Item | Classe | Nota |
|------|--------|------|
| remove apaga arquivos (TS não) | C | DEC-043 |
| update recopia (TS só manifest) | C | DEC-043 |
| publish com tarball+hash+sig (TS só meta) | C | DEC-043 |
| Sem lockfile | A | DEC-033 |
| list/info merge full | B | DEC-033 |

---

## 5. GAP (fecho)

| Item | Estado alvo |
|------|-------------|
| install/publish modules | ✅ |
| CLI lifecycle | ✅ |
| DEC-043 + docs + matriz | ✅ |
