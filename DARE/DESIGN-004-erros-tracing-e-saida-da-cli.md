# DESIGN: Erros, tracing e saída da CLI (Microplano 004)

> **Versão:** v1.0 | **Data:** 2026-07-20 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/004-erros-tracing-e-saida-da-cli.md`  
> **Referência:** ADR-002 (JSON) · `language-policy.md` · workspace 002 · CI 003  
> **Posição:** 4 de 56  
> **Arquivo:** `DARE/DESIGN-004-erros-tracing-e-saida-da-cli.md` (não substitui Designs 001–003)

---

## 1. DESCRIÇÃO

Este Design cobre a **fundação de erros, tracing e renderização de saída** do DARE CLI nativo em Rust. Hoje o binário só expõe `--help`/`--version`; falta um modelo único de `ErrorKind` → exit code, separação stdout/stderr, modo human vs JSON (ADR-002), controlo ANSI (`NO_COLOR` / TTY) e telemetria com redação de secrets.

A entrega são APIs em `dare-core` (`error`, `telemetry`) e `dare-cli` (`output` / renderer), mais testes e documentação de contrato. Quem usa são engenheiros que implementam comandos seguintes (005+) e consumidores de `--json`/CI; o usuário final ganha mensagens e códigos de saída previsíveis, sem vazamento de secrets em logs.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Erro → exit code determinístico | Mesmo `ErrorKind` produz sempre o mesmo `i32` | 100% dos kinds mapeados; testes unitários |
| O-02 | Saída human vs JSON | Renderer emite human **ou** JSON; JSON sem códigos ANSI | 0 ANSI em payloads JSON |
| O-03 | stdout ≠ stderr | Sucesso/dados em stdout; diagnósticos/erros em stderr (salvo contrato JSON de erro documentado) | Testes de captura de streams |
| O-04 | ANSI controlado | Sem cor se `NO_COLOR` set, não-TTY, ou flag explícita | Matriz TTY/NO_COLOR coberta |
| O-05 | Tracing com redaction | Patterns conhecidos (token, password, bearer, api_key) redigidos | Fixtures de redaction passam |
| O-06 | Correlation id | Contexto de execução carrega id estável por invocação | Presente em spans / campo volátil allowlist ADR-002 |
| O-07 | Desbloquear microplano 005 | Checklist MUST do 004 fechado | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Contrato estável para automação/`--json` |
| Tech Lead | Time DARE CLI Rust | ErrorKind, exit codes, ADR-002 |
| Engenheiro CLI | Time implementação | APIs clear em core/cli para comandos futuros |
| Usuário Final | Devs / agentes | Erros legíveis; exit codes úteis em scripts |
| Operações / CI | Quem roda gates | Logs sem secrets; JSON golden-friendly |
| Segurança | Tech Lead + AppSec | Redaction, sem PII em stderr/logs |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Enum `ErrorKind` (ou equivalente tipado) em `dare-core` | MUST | Kinds cobrem pelo menos: usage/cli, not found, invalid input, io, config, internal; cada um com `Display` en-US |
| RF-02 | Mapeamento `ErrorKind` → exit code | MUST | Tabela documentada; função pura `exit_code(kind) -> i32`; teste: mesmo kind ⇒ mesmo code |
| RF-03 | Erros de domínio com `thiserror` | MUST | Tipos em `dare-core` usam `thiserror`; sem `anyhow::Error` no domínio público de core |
| RF-04 | `anyhow` só na borda CLI | MUST | `dare-cli` pode usar `anyhow` para glue; conversão para `ErrorKind`/erro tipado antes de render/exit |
| RF-05 | `OutputRenderer` human / json | MUST | API tipada para sucesso e erro; modo selecionável (flag/`--json` global ou config — **fechar no Blueprint**) |
| RF-06 | JSON conforme ADR-002 | MUST | Keys de objetos em ordem lexicográfica; sem ANSI; campos voláteis (`trace_id`, etc.) na allowlist |
| RF-07 | Separação stdout / stderr | MUST | Renderer escreve mensagens humanas de erro em stderr; payload JSON de sucesso em stdout; erro JSON: política única documentada (stderr **ou** stdout+exit≠0 — Blueprint) |
| RF-08 | Controlo ANSI | MUST | Desliga se `NO_COLOR` (qualquer valor), stdout/stderr não-TTY (quando aplicável), ou `--no-color` se introduzido |
| RF-09 | Tracing subscriber | MUST | Init via `RUST_LOG` (default sensato: warn/info); spans com correlation id |
| RF-10 | Redaction em logs e erros | MUST | Função `redact(&str) -> String`; aplicada a mensagens de erro emitidas e eventos tracing |
| RF-11 | `ExecutionContext` | MUST | Struct com pelo menos: `correlation_id`, flags de output (json/color), started_at; criado no `main` e passado/clonado onde preciso |
| RF-12 | Integração mínima no binário atual | MUST | `dare --help`/`--version` continuam OK; pelo menos um caminho de erro demonstrável (ex. flag desconhecida) usa renderer + exit code mapeado |
| RF-13 | Documentação do contrato | MUST | `docs/compatibility/cli-output-and-errors.md` (ou equivalente): tabela exit codes, streams, JSON envelope mínimo |
| RF-14 | Issue/épico rastreável | SHOULD | Placeholder DEC no decision log |
| RF-15 | Golden vs TS 3.18.1 para exit codes | SHOULD | Diff intencional classificado; paridade completa de todos os códigos de domínio pode ficar parcial até comandos existirem |
| RF-16 | Envelope JSON de erro versionado (`schema_version`) | COULD | Se não houver baseline clara, adiar schema_version explícito; manter shape mínimo documentado |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Proposta inicial de exit codes (a confirmar no Blueprint)

| Exit | Significado típico | ErrorKind (proposta) |
|------|--------------------|----------------------|
| 0 | Sucesso | — |
| 1 | Erro genérico / internal | `Internal` |
| 2 | Uso inválido / CLI args | `Usage` |
| 3 | Recurso não encontrado | `NotFound` |
| 4 | Entrada/config inválida | `InvalidInput` / `Config` |
| 5 | Falha de I/O | `Io` |
| 6+ | Reservados a comandos futuros (guard, graph, …) | Documentar como “não atribuir em 004” |

> Códigos específicos de comandos posteriores (ex. guard=6) **não** são inventados aqui além da reserva; o Blueprint fecha a tabela v1 do core.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | Redaction e render no caminho quente | Overhead negligível vs I/O; sem alocações excessivas em hot path de sucesso |
| RNF-02 | Compatibilidade | ADR-002 + language-policy | JSON canónico; mensagens novas en-US |
| RNF-03 | Segurança | Secrets nunca em stdout/stderr/logs | Redaction em paths de erro e tracing |
| RNF-04 | Observabilidade | `RUST_LOG` + correlation id | Span/campo correlacionável por invocação |
| RNF-05 | Manutenibilidade | `thiserror` no domínio; `anyhow` só na borda | Clippy limpo; sem `unwrap()` em produção nesses módulos |
| RNF-06 | Testabilidade | Unit + capture de stdout/stderr | Testes sem depender de TTY real (injetar is_terminal) |
| RNF-07 | Cross-platform | Comportamento idêntico em Win/Unix para exit codes e JSON | CI 003 já cobre builds; testes unitários OS-agnostic |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar/normalizar qualquer input que entre em mensagens de erro (evitar log injection / controlo de caracteres) | OWASP A03 |
| RS-02 | Redigir tokens, passwords, API keys, Bearer, connection strings em erros e tracing | OWASP A02 |
| RS-03 | Não expor paths internos sensíveis além do necessário; sem stack traces completos em modo human default (interno só com RUST_LOG=debug) | OWASP A01 / hygiene |
| RS-04 | `cargo audit` + `cargo deny` continuam verdes após novas deps (se houver) | OWASP A06 |
| RS-05 | Sem secrets em código; redaction patterns configuráveis só via código versionado (não via env que desligue redaction em prod sem doc) | Supply chain |
| RS-06 | JSON de erro não inclui campos de secret | ADR-002 |
| RS-07 | Correlation id é UUID/random — não derivado de PII | Privacidade |
| RS-08 | Path safety avançada fica no 005 — neste ciclo só não introduzir APIs que encorajem path traversal | Escopo |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | **1.85.0** |
| Erros domínio | `thiserror` | pin workspace (002) |
| Borda CLI | `anyhow` | pin workspace |
| CLI args | `clap` | pin workspace |
| Logging | `tracing` + `tracing-subscriber` | pin workspace (+ bumps audit se preciso) |
| JSON | `serde` / `serde_json` | adicionar ao workspace se ainda não estiver; ordenação canónica (BTreeMap / serializer) |
| Cores (human) | crate leve (`anstream` / `owo-colors` / clap color) — **escolher no Blueprint** | pin |
| UUID / correlation | `uuid` v4 ou equivalente | pin |
| Testes | `assert_cmd`, captura de streams | existentes + unit |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Terminal (TTY) | I/O | stdio | Saída | human/ANSI | Time CLI |
| Env `RUST_LOG`, `NO_COLOR`, `DARE_NO_BANNER` (prep 016) | Config | env | Entrada | flags de log/cor | Time CLI |
| Advisory-db / deny | CI | HTTPS | Entrada | advisories | Time CLI |
| Baseline TS 3.18.1 | Referência | fixtures | Entrada | golden exit/JSON (parcial) | Compat |

---

## 9. RESTRIÇÕES

- **Prazo:** Pré-requisito do 005 (filesystem/path safety) e base do 016 (welcome/banner).
- **Pré-requisitos:** Microplano **002** DONE; **ADR-002** Accepted; política de idioma publicada. (003 DONE recomendado para CI.)
- **Limitações técnicas:**
  - Não implementar comandos de domínio (welcome, info, discover, …).
  - Não fechar path safety completo (005).
  - Não assinatura/SBOM/release estável.
  - Não mudar MSRV.
  - Banner figlet fica no **016** — aqui só flags/contexto que o 016 consumirá (`NO_COLOR`, TTY helpers).
- **Idioma:** mensagens novas en-US; docs de governança pt-BR.
- **Breaking:** novos exit codes públicos exigem doc; alteração futura = breaking process.

---

## 10. FORA DO ESCOPO (v1)

- Microplanos 005+ (fs seguro, processos, comandos).
- Banner ASCII / `dare welcome` (016).
- Paridade total de todos os exit codes de todos os comandos TS.
- OpenTelemetry export remoto / Jaeger.
- Structured logging para ficheiro rotativo.
- i18n / ADR-003 (normalização PT→EN em strings legadas).
- MCP / REST (051–052).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Exit codes divergem do TS sem classificação | Média | Alto | Tabela + DEC; RF-15 SHOULD com class CI |
| R-02 | ANSI vaza no JSON | Média | Alto | Teste obrigatório “JSON sem escape ANSI”; renderer separa modos |
| R-03 | Redaction incompleta (falso negativo) | Alta | Alto | Suite de fixtures com strings canónicas; documentar padrões |
| R-04 | `anyhow` vaza para core | Média | Médio | Clippy/`pub` API review; RF-03/04 |
| R-05 | Ordenação JSON não canónica | Média | Alto | Serializer ADR-002; golden unit test |
| R-06 | Detecção TTY flaky em CI | Média | Baixo | Injetar `is_terminal: bool` no contexto |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-16 priorizados (MUST vs SHOULD/COULD)
- [ ] Tabela provisória de exit codes aceite ou marcada para fechar no Blueprint
- [ ] Política erro JSON (stdout vs stderr) a fechar no Blueprint
- [ ] Crate de cores escolhido no Blueprint
- [ ] RS-01…RS-08 validados
- [ ] Fora de escopo alinhado (sem welcome/banner)
- [ ] Pré-requisitos 002 + ADR-002 confirmados
- [ ] Pronto para `/dare-blueprint` → `DARE/BLUEPRINT-004-erros-tracing-e-saida-da-cli.md`

---

## Apêndice A — Crates / paths (microplano)

| Path | Papel |
|------|-------|
| `crates/dare-core/src/error.rs` | `ErrorKind`, exit mapping, thiserror types |
| `crates/dare-core/src/telemetry.rs` | tracing init, redaction, correlation id helpers |
| `crates/dare-cli/src/output.rs` | `OutputRenderer`, human/json, stream routing |

## Apêndice B — Envelope JSON mínimo (proposta)

Sucesso (exemplo genérico — comandos futuros preenchem `data`):

```json
{
  "correlation_id": "…",
  "ok": true,
  "data": {}
}
```

Erro:

```json
{
  "correlation_id": "…",
  "ok": false,
  "error": {
    "kind": "Usage",
    "message": "…"
  }
}
```

Keys lexicográficas; `correlation_id` volátil (ADR-002 allowlist). Shape final no Blueprint.

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.
2. `/dare-blueprint` → `DARE/BLUEPRINT-004-erros-tracing-e-saida-da-cli.md`.
3. Após closeout → [`005-filesystem-seguro-e-path-safety.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/005-filesystem-seguro-e-path-safety.md).
