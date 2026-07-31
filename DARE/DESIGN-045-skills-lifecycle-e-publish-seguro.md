# DESIGN: Skills lifecycle e publish seguro (Microplano 045)

> **Versão:** v1.0 | **Data:** 2026-07-24 | **Status:** APPROVED (ciclo autorizado sem pausa humana)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/045-skills-lifecycle-e-publish-seguro.md`  
> **Referência:** Documento Mestre §4.1 · §35 · DEC-033 (044) · path safety 005 · contracts 007  
> **Posição:** 45 de 56  
> **Arquivo:** `DARE/DESIGN-045-skills-lifecycle-e-publish-seguro.md`  
> **Escopo:** `dare skill add|remove|update|publish` — install atômico, reverse-deps, tar/zip path-safe, publish com hash+assinatura. **Não** refine/patterns/graph.

---

## 1. DESCRIÇÃO

Completar o lifecycle de skills-pacote sobre o registry do microplano **044**: instalação atômica em `packages/skills/`, atualização real de conteúdo, remoção com proteção de dependências reversas, e publish de artefato `.tar.gz` verificável (SHA-256 + Ed25519). Corrigir bugs legados do TypeScript 3.18.1 (remove sem apagar arquivos; update só manifest; publish só metadados) e documentá-los como **Classe C** em **DEC-043**.

---

## 2. OBJETIVOS E MÉTRICAS

| # | Objetivo | Métrica | Meta |
|---|----------|---------|------|
| O-01 | `dare skill add` | Copia/materializa sob `packages/skills/<name>/` + upsert `.dare/skills.yml` | Unit + smoke |
| O-02 | Install atômico | Staging → rename; falha parcial não deixa half-install | Unit FS |
| O-03 | Resolve deps no add | Topo via `resolve_dependencies`; instala deps ausentes | Unit |
| O-04 | `dare skill remove` | Apaga dir + remove entrada do manifest | Unit + smoke |
| O-05 | Reverse-deps | Remove bloqueado se outra skill instalada depende | Unit |
| O-06 | `dare skill update` | Recopia conteúdo + atualiza versão no manifest | Unit |
| O-07 | Publish bundle | `.tar.gz` + `.sha256` (+ `.minisig` se chave) | Unit |
| O-08 | Validação publish | `license == MIT` e `dare_version` presente | Unit |
| O-09 | Path traversal | Entradas tar/zip com `..` / absolutas → InvalidInput | Unit |
| O-10 | CLI additive | Extender `SkillAction` / `SkillCmd` sem remover list\|info | Smoke help |
| O-11 | Docs + DEC-043 | `cli-skill.md` + DEC-043 + matriz ✅ | Docs |
| O-12 | Ralph | `cargo fmt` + clippy `-D warnings` + test `-p dare-skills -p dare-cli` | Exit 0 |

---

## 3. STAKEHOLDERS

| Papel | Interesse |
|-------|-----------|
| Product Owner | Lifecycle completo e seguro |
| Tech Lead | Extensão aditiva do enum; sem ciclo crates; path safety |
| Compat | Diffs vs TS classificados (DEC-043 Classe C nas correções) |
| Segurança | Jail de extract; sem secrets em logs; assinatura opcional |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Pri | Aceite |
|----|-----------|-----|--------|
| RF-01 | `install.rs` | MUST | API pública install/remove/update |
| RF-02 | `publish.rs` | MUST | pack + hash + sign + validate MIT/dare_version |
| RF-03 | Destino install | MUST | `packages/skills/<name>/` path-safe |
| RF-04 | Manifest projeto | MUST | Upsert/remove via `dare-contracts::SkillsManifest` (`id` = name) |
| RF-05 | Fonte add | MUST | Registry (local dir / materialize mock-remote) ou `--from` archive |
| RF-06 | Atômico | MUST | Staging sob `packages/skills/.staging-<name>/` → rename |
| RF-07 | Remove files | MUST | Apaga árvore `packages/skills/<name>` (corrige bug TS) |
| RF-08 | Reverse-deps | MUST | Scan `depends_on` de skills instaladas; bloqueia remove |
| RF-09 | Update | MUST | Recopia conteúdo da fonte + manifest (corrige bug TS) |
| RF-10 | Publish artifact | MUST | Tar.gz com paths relativos seguros + `.sha256` |
| RF-11 | Assinatura | MUST | Ed25519 via `DARE_SKILL_PRIVATE_KEY` (hex 32B); `.minisig` |
| RF-12 | Traversal | MUST | Bloquear `..`, absolutos, prefix escape em tar **e** zip |
| RF-13 | CLI | MUST | `add`/`remove`/`update`/`publish` no clap |
| RF-14 | Mensagens | MUST | en-US |
| RF-15 | Docs | MUST | `docs/compatibility/cli-skill.md` + DEC-043 |
| RF-16 | Matriz | MUST | 045 → ✅ Concluído |
| RF-17 | Hotspot | MUST | Match arms mínimos em `main.rs` |

### Superfície CLI

```text
dare skill add <name> [--from <archive>] [--version <ver>]
dare skill remove <name>
dare skill update <name> [--from <archive>]
dare skill publish <name> [--out <dir>]
```

### Contratos de disco

| Path | Papel | Mutação 045 |
|------|-------|-------------|
| `packages/skills/<name>/**` | Pacote instalado | Write/delete |
| `.dare/skills.yml` | Manifest projeto | Upsert/remove |
| `<out>/<name>-<ver>.tar.gz` | Artefato publish | Create |
| `*.sha256` / `*.minisig` | Integridade | Create |

---

## 5. REQUISITOS NÃO FUNCIONAIS

| ID | Requisito |
|----|-----------|
| RNF-01 | Sem `unwrap()` em produção |
| RNF-02 | Cross-platform (Windows rename/replace) |
| RNF-03 | Determinismo: ordenação de deps e listagens estável |
| RNF-04 | Não depender de `dare-guard` (evitar acoplamento); Ed25519 local em `publish.rs` |

---

## 6. SEGURANÇA

| ID | Controle |
|----|----------|
| RS-01 | `validate_skill_id` em todo nome |
| RS-02 | Archive entry path jail (tar + zip) |
| RS-03 | Destino final sob `ProjectRoot` via `SafeRelativePath` |
| RS-04 | Não logar `DARE_SKILL_PRIVATE_KEY` / Bearer |
| RS-05 | Publish rejeita license ≠ MIT |

---

## 7. FORA DE ESCOPO

- `dare refine` / `dare patterns` / `dare graph`
- Lockfile / integrity no install (permanece ausente — DEC-033)
- Alterar schema breaking de `SkillsManifest` sem ADR
- Wire minisign completo (mesmo padrão Classe B do guard: Ed25519 dare)

---

## 8. DECISÕES → BLUEPRINT

| # | Tema | Default |
|---|------|---------|
| D-01 | Correção bugs TS | Classe **C** (corrigir) — DEC-043 |
| D-02 | Archive libs | `tar` + `flate2` + `zip` (workspace pins) |
| D-03 | Assinatura | Ed25519 dalek; env `DARE_SKILL_PRIVATE_KEY` |
| D-04 | Mock content | Materializar `skill.yml` + `SKILL.md` stub |
| D-05 | DEC | **DEC-043** only |

---

## 9. CRITÉRIOS DE ACEITE

- [ ] Remove apaga arquivos corretos
- [ ] Update recopia conteúdo
- [ ] Publish produz artefato + hash (+ sig se chave)
- [ ] Extração maliciosa bloqueada (tar e zip)
- [ ] fmt / clippy `-D warnings` / test `-p dare-skills -p dare-cli`
- [ ] Matriz 045 Concluído; DEC-043; docs atualizados
