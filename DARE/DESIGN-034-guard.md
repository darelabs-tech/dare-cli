# DESIGN: Guard — unicode, injection, proveniência e preflight (Microplano 034)

> **Versão:** v1.0 | **Data:** 2026-07-22 | **Status:** APPROVED (ciclo autorizado sem pausa)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/034-guard.md`  
> **Referência:** Documento Mestre §5.4 Guard · §30 Ciclo 12 · exit **6** · Microplanos **005**/**006** · stub `guard_preflight_stub` em **030**  
> **Posição:** 34 de 56  
> **Arquivo:** `DARE/DESIGN-034-guard.md`  
> **Escopo:** crate **`dare-guard`**, `dare guard`, `assets/rules/scan-rules.json`, exit **6**, preflight real do `--agent`. **Não** hooks/steering (**048**).

---

## 1. DESCRIÇÃO

Implementar o pipeline de segurança Guard em três camadas (Unicode → Scan injection → Proveniência/assinatura) com veredito `PASS|WARN|FAIL`, superfície CLI `dare guard`, e integração como preflight obrigatório de `dare execute --agent` (substituir stub 030). FAIL → exit **6**; agent não inicia.

---

## 2. OBJETIVOS E MÉTRICAS

| # | Objetivo | Meta |
|---|----------|------|
| O-01 | Detectar zero-width, bidi, variation selectors, tags, homoglyphs | Unit corpus |
| O-02 | Modos `strip` e `block` | Unit |
| O-03 | Carregar `scan-rules.json` (4 built-in + override path) | Unit + asset |
| O-04 | Regras injection + evidência redigida | Unit |
| O-05 | Proveniência human/agent/external + trustedPaths | Unit |
| O-06 | Assinar/verificar Ed25519 (`.minisig`) | Unit |
| O-07 | Exit code **6** em FAIL | CLI smoke |
| O-08 | Preflight agent: FAIL → não inicia, exit 6 | Integração |
| O-09 | Docs + DEC-035 + matriz 034 Concluído | Artefatos |

---

## 3. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade |
|----|-----------|------------|
| RF-01 | Crate `dare-guard` (workspace member) | MUST |
| RF-02 | Unicode detect + strip/block | MUST |
| RF-03 | Asset `assets/rules/scan-rules.json` + env `DARE_GUARD_SCAN_RULES_PATH` | MUST |
| RF-04 | Scan regex 4 regras built-in; evidência redigida | MUST |
| RF-05 | Proveniência + `trustedPaths` (config `guard.extra`) | MUST |
| RF-06 | Control artifacts exigem assinatura se `signing.enabled` | MUST |
| RF-07 | `dare guard --sign` com `DARE_GUARD_PRIVATE_KEY` | MUST |
| RF-08 | CLI: `[target]`, `--staged`, `--all`, `--unicode`, `--strict`, `--format`, `--fail-on`, `--sign` | MUST |
| RF-09 | `ErrorKind::GuardFail` → exit **6** | MUST |
| RF-10 | Substituir `guard_preflight_stub` por preflight real | MUST |
| RF-11 | Docs `cli-guard.md` + DEC-035; atualizar `cli-execute-agent.md` | MUST |
| RF-12 | Capability `dare-guard` → `cli_commands: ["guard"]` | SHOULD |
| RF-13 | Mensagens domínio en-US | MUST |
| RF-14 | Path safety (005); sem shell concatenado (006) | MUST |

### Fora de escopo

- Hooks / steering (**048**)
- Drivers agent reais (**031**)
- Dashboard telemetry deep (**050**)

---

## 4. REQUISITOS NÃO-FUNCIONAIS / SEGURANÇA

| ID | Requisito |
|----|-----------|
| RNF-01 | Evidências de match nunca incluem secrets em claro (redact) |
| RNF-02 | Chave privada só via env; nunca logada |
| RNF-03 | Cap de leitura por ficheiro (ex. 1 MiB) |
| RS-01…08 | Alinhar path safety, redact, audit deps |

---

## 5. STACK

- Rust 1.85 / workspace
- `regex` (scan)
- `ed25519-dalek` (assinatura; formato wire dare-guard Ed25519 em `.minisig` — DEC)
- `dare-core` (ErrorKind, ProjectRoot, SafeCommand, redact)
- `serde`/`serde_json`

---

## 6. CRITÉRIOS DE ACEITE

- [ ] Corpus malicioso detectado (unicode + injection)
- [ ] Evidências redigidas
- [ ] Assinatura inválida falha
- [ ] Agent não inicia após FAIL (exit 6)
- [ ] `cargo fmt --check`, `clippy -D warnings`, `test --workspace`
- [ ] Matriz 034 → Concluído
- [ ] DEC-035 + docs

---

## 7. RISCOS

| Risco | Mitigação |
|-------|-----------|
| Conflito `main.rs` / `dare-agent` | Branch isolada; wire fino |
| Falso positivo injection | Severidade warn vs fail; `--fail-on` |
| Formato minisign ≠ wire TS | DEC Classe B: Ed25519 dare-guard |

---

## 8. APROVAÇÃO

Ciclo completo autorizado pelo usuário (Design→Execute sem pausa). Blueprint pode congelar trade-offs e executar.
