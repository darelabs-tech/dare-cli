# DESIGN: Pilotos, shadow tests e release candidate (Microplano 055)

> **Versão:** v1.0 | **Data:** 2026-07-31 | **Status:** APPROVED (blueprint gerado)  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/055-pilotos-shadow-tests-e-release-candidate.md`  
> **Referência:** Documento Mestre §46 (canais alpha/beta/rc/stable) · §47.2 passos 2–4 (shadow, freeze TS, RC + bloqueio de contrato) · §47.3 rollback · `dare-parity` / DEC-055 (**054**) · `fixtures-inventory.md` · `breaking-change-process.md` · baseline `@dewtech/dare-cli@3.18.1` · self-update **053** · próximo **056** (cutover stable)  
> **Posição:** 55 de 56  
> **Arquivo:** `DARE/DESIGN-055-pilotos-shadow-tests-e-release-candidate.md`  
> **Escopo deste ciclo:** selecionar projetos piloto · shadow (Rust em paralelo **sem mutar** quando aplicável) · comparar outputs/operação diária · coletar incidentes/gaps · feature-freeze TypeScript exceto segurança · publicar **RC** · bloquear mudanças de contrato · validar rollback operacional · docs em `docs/pilot` + `docs/release-candidate` + **DEC-056**.  
> **Não** cutover stable / npm legacy / v1.0.0 oficial (**056**). **Não** novas features de produto. **Não** mudar Classe A sem ADR. DEC proposto: **DEC-056** (DEC-055 = hardening **054**).

---

## 1. DESCRIÇÃO

Validar o DARE CLI Rust em **projetos reais** e publicar um **release candidate** com contratos congelados, antes do cutover oficial (**056**).

O problema: a suite de paridade/security (**054**) prova comportamento em fixtures; ainda falta provar fluxos diários em projetos piloto, operar shadow sem risco de corromper working trees, e ter um canal **rc** instalável com rollback testado por operadores. Quem usa: Product Owner, Release, e times piloto. Entrega verificável: inventário e playbooks em `docs/pilot/`, pacote/processo RC em `docs/release-candidate/`, log de incidentes/gaps classificados, freeze TS documentado, artefato RC + smoke, e **DEC-056**.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Projetos piloto selecionados | Lista versionada em `docs/pilot/pilots.md` (id, stack, owner, OS) | ≥ **3** pilotos (Linux + macOS + Windows cobertos no conjunto) |
| O-02 | Shadow sem mutação | Modo shadow documentado + script/checklist que roda Rust **read-only** / worktree cópia | 0 write no projeto original em shadow |
| O-03 | Comparar outputs diários | Relatório por piloto: exit/stdout/tree/state vs TS ou baseline nativa classificada | Cada fluxo MUST com pass ou gap classificado A/B/C/D |
| O-04 | Incidentes / gaps | `docs/pilot/incidents.md` (P0–P3) | **0** P0/P1 abertos no close |
| O-05 | Feature freeze TS | Anúncio + política em `docs/release-candidate/typescript-freeze.md` | Só patches de segurança no legado TS |
| O-06 | Publicar RC | Tag/canal **`v4.0.0-rc1`** (prerelease GitHub) + binários + checksums + assinatura alinhada ADR-008 | Artefato instalável + smoke `--version` |
| O-07 | Bloquear contrato | Gate CI/docs: Classe A só via `breaking-change-process.md` | PR que altera Classe A sem ADR → bloqueado / checklist FAIL |
| O-08 | Rollback operacional | Runbook + exercício documentado (`dare self rollback` e/ou reinstalar artefato anterior) | ≥1 drill assinado por operador |
| O-09 | Compat residual | Diffs novos → `parity-diff-log` / ADR | 0 diferença sem classificação |
| O-10 | Docs + DEC-056 | `docs/pilot/**`, `docs/release-candidate/**`, DECISION-LOG | Review |
| O-11 | Ralph close | `cargo fmt --check`, clippy, test, `cargo audit` | Exit 0; 0 CVE HIGH/CRITICAL |
| O-12 | Cross-platform RC | Smoke RC em Linux, macOS, Windows (CI ou checklist piloto) | 3 OS OK ou gap documentado |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs | Go/no-go para 056 |
| Tech Lead | DARE CLI Rust | DEC-056; freeze de contrato; critérios P0/P1 |
| Release / Ops | CI + GitHub Releases | Tag RC, checksums, assinatura, rollback |
| Times piloto | Owners dos projetos | Shadow sem risco; gaps acionáveis |
| Segurança | — | Freeze TS exceto security; audit RC |
| Compat | Baseline / parity | Diffs classificados; sem regressão 054 |
| Usuário final (beta) | Early adopters | Instalar RC e reportar |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Inventário de pilotos | MUST | `docs/pilot/pilots.md` com ≥3 entradas: `pilot_id`, stack, repo path/URL (ou fixture espelho), OS, owner, fluxos MUST |
| RF-02 | Critérios de seleção | MUST | Documento lista critérios (brownfield real ou fixture materializada do inventário 054; ≥1 harness IDE se possível) |
| RF-03 | Playbook shadow | MUST | `docs/pilot/shadow-playbook.md`: como clonar/copiar, env, comandos Rust vs TS, **proibição de write** no original |
| RF-04 | Modo não-mutante | MUST | Shadow usa cópia/worktree **ou** flags `--dry-run` / read-only onde o CLI já oferece; writes só na cópia |
| RF-05 | Matriz de fluxos | MUST | Por piloto: tabela fluxo → comando(s) → eixos comparados (reusar dimensões 054) → resultado |
| RF-06 | Comparação automatizável | SHOULD | Script ou target que invoca `dare-parity` / golden contra cwd do piloto-cópia; se inviável, checklist assinado com evidências anexadas |
| RF-07 | Log de incidentes | MUST | `docs/pilot/incidents.md`: id, severidade P0–P3, piloto, repro, status, classificação compat |
| RF-08 | Severidade | MUST | P0 = data loss / security / unusable CLI; P1 = fluxo MUST quebrado; P2 = workaround; P3 = cosmético |
| RF-09 | Gate close | MUST | Close 055 bloqueado se qualquer P0/P1 `open` |
| RF-10 | Freeze TypeScript | MUST | `docs/release-candidate/typescript-freeze.md` + nota em CHANGELOG/Release: TS só security fixes |
| RF-11 | Canal RC | MUST | Publicar prerelease GitHub tag **`v4.0.0-rc1`** (major 4 após baseline npm **3.18.1**) com assets ADR-008 |
| RF-12 | Artefatos RC | MUST | Binários + `SHA256SUMS` + `.sig` (cosign); SBOM SHOULD |
| RF-13 | Release notes RC | MUST | `docs/release-candidate/RELEASE-NOTES.md` (known issues, freeze, como instalar/rollback) |
| RF-14 | Bloqueio de contrato | MUST | Checklist em `docs/release-candidate/contract-freeze.md` + referência a `breaking-change-process.md`; CI job ou CODEOWNERS note SHOULD |
| RF-15 | Sem semver major silenciosa | MUST | Mudança Classe A no RC exige ADR Accepted **antes** do merge |
| RF-16 | Rollback drill | MUST | `docs/release-candidate/rollback-drill.md` preenchido: passos, operador, data, resultado OK |
| RF-17 | Rollback caminhos | MUST | Cobrir: (a) `dare self rollback` se self-update disponível no RC; (b) reinstalar tag anterior documentada |
| RF-18 | Smoke pós-install RC | MUST | `dare --version`, `dare info`, 1 comando help crítico por OS no smoke |
| RF-19 | Diffs novos | MUST | Qualquer gap de piloto sem fix → entrada `parity-diff-log` ou incidente classificado |
| RF-20 | Capability | COULD | Sem capability nova por padrão (igual 054); Blueprint confirma |
| RF-21 | Docs paths | MUST | Árvore `docs/pilot/` + `docs/release-candidate/` criada e linkada em `docs/compatibility/README.md` ou índice release |
| RF-22 | DEC-056 | MUST | Append-only DECISION-LOG; **não** editar DEC-055 |
| RF-23 | Matriz 000A | MUST | Microplano 055 → Concluído ao fechar |
| RF-24 | Reuso 054 | MUST | Não reimplementar golden/security; consumir `dare-parity` / docs existentes |
| RF-25 | Mensagens en-US | MUST | Release notes / playbooks operacionais voltados a usuário em inglês se forem UX pública; docs DARE PT OK |
| RF-26 | Consentimento piloto | MUST | Owners listados; sem expor secrets/PII de projetos reais nos commits (redact paths sensíveis) |

> **MUST** · **SHOULD** · **COULD**

### Superfície operacional (proposta Analyst — Blueprint confirma)

```text
docs/pilot/
  pilots.md
  shadow-playbook.md
  incidents.md
  results/<pilot_id>/   # evidências (logs redacted)
docs/release-candidate/
  RELEASE-NOTES.md
  typescript-freeze.md
  contract-freeze.md
  rollback-drill.md
# Opcional:
scripts/pilot-shadow.ps1 | .sh   # orquestra cópia + comandos read-only
```

### Princípio shadow (inegociável)

| Regra | Detalhe |
|-------|---------|
| Isolamento | Nunca escrever no working tree original do piloto |
| Evidência | Logs/artefatos só sob `docs/pilot/results/` ou temp descartável |
| Classificação | Gap → incidente P* + classe A/B/C/D |
| Segurança | Mesmos invariantes path/argv/redact do produto |

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Disponibilidade | Canal RC publicável e baixável | URL Release estável por tag |
| RNF-02 | Confiabilidade | Shadow determinístico o bastante para repetir | Re-run playbook → mesmo veredicto |
| RNF-03 | Performance | Shadow não exige otimização nova | Usa baselines 054; só reporta outliers |
| RNF-04 | Segurança | RC sem CVE HIGH/CRITICAL | `cargo audit` no close |
| RNF-05 | Observabilidade | Incidentes e resultados versionados no git | Paths § RF-21 |
| RNF-06 | Manutenibilidade | Playbooks curtos (< ~200 linhas cada) | Review PO/TL |
| RNF-07 | Portabilidade | Pilotos cobrem 3 OS no conjunto | O-01 / O-12 |
| RNF-08 | Tempo | Janela de shadow documentada | Blueprint sugere duração mínima (ex. ≥5 dias úteis) ou N ciclos diários |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Entradas de scripts/playbooks validadas (paths, allowlist de comandos) | OWASP A03 |
| RS-02 | Sem secrets/PII de projetos piloto em commits; redact em logs anexados | OWASP A02 / CI-012 |
| RS-03 | Shadow não eleva privilégio; writes só em cópia/jail | OWASP A01 / path-safety |
| RS-04 | `cargo audit` limpo no close do RC | OWASP A06 |
| RS-05 | Tokens CI/Release só via secrets do GitHub Actions / env | Supply chain |
| RS-06 | Assinatura/checksum dos assets RC verificáveis | ADR-008 / CI-014 |
| RS-07 | Spawn argv-only em scripts de shadow | CI-011 |
| RS-08 | Freeze TS permite **somente** fixes de segurança documentados | §47.2 |
| RS-09 | Rollback drill não deixa binário RC “meio instalado” | §47.3 / 053 |
| RS-10 | Consentimento/ownership dos pilotos registrado | RF-26 |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| CLI sob validação | `dare-cli` + workspace Rust | `rust-version` 1.88 |
| Paridade | `dare-parity` + `tests/**` | do **054** |
| Self-update / rollback | `dare-self` | do **053** |
| Release | GitHub Releases + CI `release.yml` | existente |
| Assinatura | cosign verify-blob | ADR-008 |
| Docs | Markdown sob `docs/pilot`, `docs/release-candidate` | — |
| Baseline legado | `@dewtech/dare-cli@3.18.1` | comparação shadow |
| Audit | `cargo-audit` | 0.22.x (CI) |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| GitHub Releases | Distribuição RC | HTTPS | Saída | Binários, SUMS, sig | Release |
| Projetos piloto | Validação | FS / git local | Entrada (cópia) | Outputs CLI | Owners piloto |
| npm baseline 3.18.1 | Comparação shadow | CLI local | Entrada | exit/stdout/tree | Compat |
| cosign / Sigstore | Verify | argv local | Entrada | sig status | Segurança |
| cargo audit / RustSec | Advisory | HTTPS | Entrada | CVEs | Segurança |

---

## 9. RESTRIÇÕES

- **Prazo:** Microplano 55/56 — bloqueia cutover **056**.
- **Pré-requisito:** **054** concluído (golden/security/baselines).
- **Orçamento:** Sem infra nova além de GitHub Actions/Releases já usados.
- **Limitações técnicas:** Sem mutar projetos originais; sem mudança Classe A sem ADR; RC ≠ stable.
- **Regulatórias:** Sem PII/secrets de clientes nos artefatos git.
- **DEC:** **DEC-056** (055 já usado por hardening 054).
- **Canal:** RC é prerelease tag **`v4.0.0-rc1`** (não `0.1.0-*`); default `dare self` beta permanece até 056; instalar RC via `--version` explícito.

---

## 10. FORA DO ESCOPO (v1)

- Publicar **v1.0.0 stable** e tornar Rust recomendado oficial → **056**.
- Mover npm TypeScript para `legacy` / arquivar legado → **056**.
- Janela longa de suporte pós-cutover e relatório final de compatibilidade → **056**.
- Novas features / novos comandos CLI.
- Reescrever harness `dare-parity` (só consumir).
- Otimizações de performance além de observar baselines 054.
- Docker packaging / Scoop.
- Exigir N grandes de pilotos comerciais secretos (usar fixtures reais materializadas se owners indisponíveis — documentar).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Shadow escreve no projeto original | Média | Alto | Playbook + cópia obrigatória; checklist “dry-run first” |
| R-02 | Pilotos insuficientes / sem 3 OS | Alta | Médio | Aceitar fixtures inventário 054 como piloto “synthetic”; marcar no inventário |
| R-03 | P0 tarde demais | Média | Alto | Severidade clara; freeze close até 0 P0/P1 |
| R-04 | RC confundido com stable | Média | Alto | Tag/prerelease + docs; help/install notes |
| R-05 | Contrato muda durante RC | Média | Alto | contract-freeze + breaking-change-process |
| R-06 | Rollback drill falha em Windows | Média | Médio | Drill explícito Win + `dare self rollback` |
| R-07 | Vazamento de path/PII de piloto | Média | Alto | Redact; paths genéricos nos commits |
| R-08 | Confusão DEC-055 vs 056 | Baixa | Médio | Header deste DESIGN; append-only |
| R-09 | Janela shadow curta demais | Média | Médio | Blueprint define mínimo de ciclos/dias |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] ≥3 pilotos / política de synthetic fixtures aceita
- [ ] Shadow sem mutação no original aceito
- [ ] Severidade P0–P3 e gate “0 P0/P1” aceitos
- [ ] Freeze TS (só security) aceito
- [ ] Tag RC **`v4.0.0-rc1`** (pós-baseline 3.18.1) alinhada
- [ ] Rollback drill obrigatório aceito
- [ ] DEC id **056** confirmado (055 = hardening)
- [ ] Fora de escopo 056 alinhado com PO
- [ ] Segurança RS-01..10 revisada pelo Tech Lead
- [ ] Aprovar para `/dare-blueprint` → `DARE/BLUEPRINT-055-pilotos-shadow-tests-e-release-candidate.md`

---

## Notas Analyst → PM (passagem única)

### Analyst

| Kind | Item | Marcação |
|------|------|----------|
| scope | Pilotos + shadow + RC + freeze contrato/TS; sem cutover 056 | 🟢 Microplano 055 · Mestre §46–47 |
| ambiguity | Pilotos = repos externos vs fixtures materializadas | 🟡 proposta: preferir reais; se faltarem, synthetic do inventário 054 contam com label `synthetic: true` |
| ambiguity | Tag RC exata | 🟢 **`v4.0.0-rc1`** (major após npm 3.18.1; self via `--version`) |
| ambiguity | Script `pilot-shadow` MUST vs playbook manual | 🟡 proposta: playbook MUST; script SHOULD |
| ambiguity | Capability nova | 🟡 proposta: **não** (só docs) |
| ambiguity | Duração mínima da janela shadow | 🔴 Blueprint (ex. ≥3 execuções ou ≥5 dias) |
| gap | `docs/pilot` e `docs/release-candidate` ainda não existem | 🟢 criar neste microplano |
| gap | Owners/consentimento dos pilotos | 🔴 PO preenche na execução |
| gap | CI job que bloqueia Classe A automaticamente | 🟡 SHOULD; checklist MUST |

### PM

- Aceite v1: ≥3 pilotos documentados; shadow isolado; 0 P0/P1; freeze TS; RC publicado com notes + rollback drill OK; DEC-056; Ralph/audit verdes.
- Preferir **não** inventar CLI `dare pilot` neste ciclo — docs + scripts leves bastam.
- RC deixa explícito que **não** é cutover; 056 faz stable/legacy.

---

## Próximas etapas

1. Revisar e aprovar este Design (pilotos vs synthetic, tag **`v4.0.0-rc1`**, duração shadow, script SHOULD).
2. Quando aprovado, rodar `/dare-blueprint` com `@DARE/DESIGN-055-pilotos-shadow-tests-e-release-candidate.md`.
3. Em seguida `/dare-tasks` → executar DAG 055.
