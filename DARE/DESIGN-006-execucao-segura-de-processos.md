# DESIGN: Execução segura de processos (Microplano 006)

> **Versão:** v1.0 | **Data:** 2026-07-20 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/006-execucao-segura-de-processos.md`  
> **Referência:** Microplanos 002+004 (+005 path/cwd) · Documento Mestre §5.5 safe-spawn · exit 124 · `tokio::process`  
> **Posição:** 6 de 56  
> **Arquivo:** `DARE/DESIGN-006-execucao-segura-de-processos.md` (não substitui Designs 001–005)

---

## 1. DESCRIÇÃO

Este Design cobre o **executor de processos seguro e cancelável** do DARE CLI nativo em Rust, substituto do `child_process` / `safe-spawn` do baseline TypeScript 3.18.1. Verificação (Ralph), hooks, adapters de agentes e enrichment spawnam CLIs externos; sem argv separado, allowlist de env, timeout e kill de árvore, o CLI vaza secrets, fica preso em hangs ou deixa órfãos.

A entrega é a API em `dare-core` (`process.rs`): `SafeCommand` (argv, sem shell), sanitização de environment, captura limitada de stdout/stderr, timeout com código **124**, cancelamento com kill da árvore, erro normalizado para executável ausente, e um **mock process runner** para testes determinísticos. Quem usa são os microplanos 029+ (Ralph), 031 (drivers), 048 (hooks) e 049 (verificação); o usuário final ganha gates que falham de forma previsível sem hang eterno nem vazamento de `TOKEN`/`SECRET` em logs.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Zero shell concatenado | Nenhuma API pública aceita `cmd`/`sh -c` string única | 100% spawns via argv |
| O-02 | Env sem secrets óbvios | Vars com `SECRET`/`TOKEN`/`KEY`/`PASSWORD` no nome removidas (ou deny) | Asserts em suite |
| O-03 | Timeout → 124 | Processo que excede deadline reporta exit **124** | Teste com sleep/fixture |
| O-04 | Sem órfãos | Após timeout/cancel, filhos da árvore não ficam vivos | Assert pós-kill (Unix + Win) |
| O-05 | Cap stdout/stderr | Capture truncada no limite documentado (baseline **4000** chars) | Assert comprimento + flag `truncated` |
| O-06 | Executável ausente | Erro tipado estável (en-US), não panic | Teste path inexistente |
| O-07 | Mock runner | Testes unitários sem binário real quando injetado | Suite green sem spawn |
| O-08 | Desbloquear Ralph / verificação | Checklist MUST do 006 fechado | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Gates Ralph/verificação sem hang nem leak |
| Tech Lead | Time DARE CLI Rust | Contrato `SafeCommand` / exit 124 / tokio vs sync |
| Engenheiro CLI | Time implementação | API reutilizável para 029–031, 048–049 |
| Usuário Final | Devs / agentes | Timeout previsível; logs sem secrets |
| Segurança | Tech Lead + AppSec | Injection via shell; env leak (OWASP A03/A02) |
| Operações / CI | CI 003 matrix | Comportamento Win + Unix (kill tree) |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Tipo `SafeCommand` | MUST | Constrói com programa + `argv` separado; **proíbe** shell (`shell: false` equivalente); sem API que concatene string de comando |
| RF-02 | `cwd` sob jail | MUST | Working directory via `ProjectRoot` + path seguro (005); rejeita cwd fora do root |
| RF-03 | Environment allowlist / denylist | MUST | Política fechada no Blueprint: **remover** vars cujo nome case-insensitive contém `SECRET`, `TOKEN`, `KEY`, `PASSWORD` (baseline TS); opcional allowlist explícita de prefixes (`PATH`, `HOME`, `LANG`, `DARE_*`, …) — **fechar no Blueprint** |
| RF-04 | Captura stdout/stderr | MUST | Buffers capturados; limite default **4000** chars (paridade TS); excesso truncado com indicação observável (`truncated: true` ou sufixo documentado) |
| RF-05 | Timeout | MUST | Deadline configurável; ao estourar: sinal de terminação → resultado com **exit code 124** (paridade GNU timeout / baseline) |
| RF-06 | Kill de árvore | MUST | Em timeout **e** cancelamento: processo raiz + descendentes terminados; sem órfãos verificáveis em teste |
| RF-07 | Cancelamento | MUST | API de cancel (token/`Abort`/`CancellationToken` — **escolher no Blueprint**) interrompe spawn em curso e aplica RF-06 |
| RF-08 | Executável ausente | MUST | `CoreError` kind estável (`NotFound` ou `Io` documentado); mensagem en-US sem path com secrets; **não** panic |
| RF-09 | Resultado estruturado | MUST | Tipo `ProcessOutput` (nome final no Blueprint) com: `exit_code`, `stdout`, `stderr`, flags de truncate/timeout/cancelled |
| RF-10 | Trait / mock runner | MUST | Abstração `ProcessRunner` (ou equivalente) + implementação mock injetável; testes unitários sem depender de `sleep`/`true` quando usam mock |
| RF-11 | Integração erros 004 | MUST | Falhas de spawn/validação → `CoreError` + `redact`; **124** é código do *filho/resultado*, não conflita com exit 1–5 do próprio binário `dare` salvo política documentada no Blueprint |
| RF-12 | Integração path 005 | MUST | Programa e cwd resolvidos/validados sem escape de root quando forem paths relativos ao projeto |
| RF-13 | Documentação | MUST | `docs/compatibility/process-safety.md`: argv, env policy, 124, truncate, kill-tree, mock |
| RF-14 | DEC no decision log | SHOULD | DEC-007 (tokio vs sync, kill-tree Win, env policy, truncate) |
| RF-15 | Paridade golden TS | SHOULD | Truncate 4000, remove SECRET/TOKEN/KEY/PASSWORD, timeout→124 classificados vs baseline |
| RF-16 | Streaming live de stdout | COULD | Fora do 006; captura buffered basta (watch/stream em microplanos posteriores se necessário) |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contrato de saída de processo (observável)

| Campo | Semântica |
|-------|-----------|
| `exit_code` | Código do processo; **124** = timeout |
| `stdout` / `stderr` | Texto capturado (UTF-8 lossy ou policy Blueprint); truncado se > limite |
| `timed_out` | `true` quando RF-05 aplicou |
| `cancelled` | `true` quando RF-07 aplicou |

Alteração de semântica pública de 124 / truncate ⇒ ADR + nota de compatibilidade.

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Segurança | Nenhum spawn via shell; env sanitizado antes do exec | 0 usos de `cmd.exe /c` / `sh -c` nas APIs deste módulo |
| RNF-02 | Performance | Overhead de spawn aceitável para gates Ralph | Timeout default alinhado a usos futuros (ex. verificação); não bloquear event loop se async |
| RNF-03 | Compatibilidade | Linux, macOS, Windows | Kill-tree testado ou documentado por `cfg` |
| RNF-04 | Observabilidade | Spawns em `tracing` (programa + argc, **sem** env values sensíveis) | Redact em erros/logs |
| RNF-05 | Manutenibilidade | `dare-core/src/process.rs` (+ submódulos se preciso) | Clippy limpo; sem `unwrap` em prod |
| RNF-06 | Testabilidade | Mock + fixtures reais mínimas (`true`/`cmd /c echo`/sleep curto) | Suite CI matrix |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar programa, argv e cwd antes do spawn (sem NUL perigosos; cwd no jail) | OWASP A03 |
| RS-02 | Não logar valores de env; stdout/stderr em erros passam por `redact` quando expostos | OWASP A02 |
| RS-03 | Não elevar privilégios; não herdar env completo sem sanitização | OWASP A01 / hygiene |
| RS-04 | `cargo audit` + `cargo deny` verdes após novas deps (ex. tokio) | OWASP A06 |
| RS-05 | Secrets só via env do host já filtrado — nunca hardcoded em fixtures de processo | Supply chain |
| RS-06 | Proibir shell concatenado / string interpolation de comando | Command injection |
| RS-07 | Timeout + kill-tree obrigatórios em spawns de longa duração da API pública | DoS / hang |
| RS-08 | Truncar captura para limitar memória e vazamento acidental em logs | Resource / A02 |
| RS-09 | Mock runner não executa binários reais — evita side effects em unit tests | Isolamento |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | 1.85.0 | pin existente |
| Erros / redact | `CoreError`, `redact` (004) | existente |
| Path / cwd | `ProjectRoot`, `SafeRelativePath` (005) | existente |
| Processos | **`tokio::process`** (proposta Documento Mestre) **ou** `std::process` + thread — **fechar no Blueprint** | pin no Blueprint |
| Cancel | `tokio_util::sync::CancellationToken` / canal — **A confirmar** | Blueprint |
| Kill tree | crate (`kill_tree` / `sysinfo` / Job Object Win) — **A confirmar** | Blueprint |
| Testes | `tempfile`, mock runner, binários OS | — |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| OS process API | Exec | spawn/wait/kill | Saída+entrada | argv, env filtrado, streams | Time CLI |
| CI runners Win/Unix | Test | GHA | Entrada | Suite kill/timeout | Time CLI |
| Baseline TS 3.18.1 safe-spawn | Referência | fixtures | Entrada | 4000 / 124 / denylist | Compat |
| Futuros Ralph/hooks/agents | Consumidores | API Rust | Entrada | `SafeCommand` / runner | Microplanos 029+ |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** Microplanos **002** e **004** DONE (MUST do plano). **005** DONE recomendado/obrigatório na prática para `cwd` jail.
- **Prazo:** Bloqueia verificação/Ralph (029+), hooks (048) e drivers que spawnam CLI (031).
- **Limitações:**
  - Não implementar Ralph Loop completo, hooks, nem agent drivers neste ciclo.
  - Não sandbox OS (seccomp, AppContainer) — só safe-spawn.
  - Não alterar exit codes públicos 1–5 do binário `dare` sem ADR; 124 é do *resultado do filho*.
  - Não streaming interativo TTY completo.
  - Não PTY / pseudo-terminal.
- **Idioma:** mensagens en-US; docs pt-BR.
- **Breaking:** mudar 124 / truncate / denylist ⇒ ADR + DEC.

---

## 10. FORA DO ESCOPO (v1)

- Microplanos 007+ (contratos persistidos, config, comandos de produto).
- Aspectos de verificação (`build|test|lint|…`) e baseline `.dare/verification/` (049).
- Drivers reais de agentes e worktrees (030–031).
- Sandbox / container isolation / network policy por processo.
- Job control interativo e attach a TTY do usuário.
- Rate limiting de spawns concorrentes (fanout — posteriores).
- Replicar bugs conhecidos do TS sem classificação (Classe B/C no compat doc).

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Kill-tree no Windows incompleto | Alta | Alto | Job Objects ou crate madura; testes `cfg(windows)`; documentar gaps |
| R-02 | `tokio` puxa runtime para `dare-core` | Média | Médio | Feature `process-async` **ou** `std::process` + timeout em thread — DEC-007 |
| R-03 | Denylist incompleta (ex. `API_KEY` vs `KEY`) | Média | Alto | Baseline + testes de nomes; documentar heurística; allowlist opcional |
| R-04 | Truncate 4000 quebra JSON parcial em stdout | Média | Médio | Flag `truncated`; consumidores não parseiam cego; DEC |
| R-05 | Exit 124 confunde com ErrorKind CLI | Baixa | Médio | Separar `ProcessOutput.exit_code` vs `dare` process exit; docs |
| R-06 | SIGTERM ignorado por filho teimoso | Média | Médio | Escalation SIGKILL / TerminateProcess após grace period (Blueprint) |
| R-07 | Paridade TS diverge em encoding stderr | Média | Baixo | UTF-8 lossy + classificação compat |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-16 priorizados
- [ ] Política env (denylist vs allowlist) aceite ou marcada para Blueprint
- [ ] Async (`tokio`) vs sync fechado ou deferido ao Blueprint
- [ ] Estratégia kill-tree Windows aceite como risco consciente
- [ ] Semântica exit **124** vs exit codes do binário `dare` clara
- [ ] RS-01…RS-09 validados
- [ ] Fora de escopo alinhado (sem Ralph/hooks/agents)
- [ ] Pré-requisitos 002+004 (e 005 para cwd) confirmados
- [ ] Pronto para `/dare-blueprint` → `DARE/BLUEPRINT-006-execucao-segura-de-processos.md`

---

## Apêndice A — Crates / paths (microplano)

| Path | Papel |
|------|-------|
| `crates/dare-core/src/process.rs` | `SafeCommand`, runner, timeout, mock |
| `crates/dare-core/src/path.rs` / `fs/` | cwd / paths (005) — consumo, não reimplementar |

## Apêndice B — Comportamento baseline (proposta)

Alinhamento SHOULD com TS `safe-spawn`:

| Aspecto | Valor |
|---------|-------|
| Shell | Desligado (`argv` only) |
| Env | Remove nomes com `SECRET` \| `TOKEN` \| `KEY` \| `PASSWORD` |
| Truncate | 4000 caracteres por stream |
| Timeout | Terminação → exit **124** |

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `DARE/BLUEPRINT-006-execucao-segura-de-processos.md`.  
3. Após closeout → [`007-contratos-persistidos.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/007-contratos-persistidos.md).
