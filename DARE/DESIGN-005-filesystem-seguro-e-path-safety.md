# DESIGN: Filesystem seguro e path safety (Microplano 005)

> **Versão:** v1.0 | **Data:** 2026-07-20 | **Status:** DRAFT  
> **Fonte:** `DARE-RUST-MICRO-PLANOS/005-filesystem-seguro-e-path-safety.md`  
> **Referência:** Microplanos 002+004 · Documento Mestre (path-safety, Windows) · `disk-and-json-policy.md`  
> **Posição:** 5 de 56  
> **Arquivo:** `DARE/DESIGN-005-filesystem-seguro-e-path-safety.md` (não substitui Designs 001–004)

---

## 1. DESCRIÇÃO

Este Design cobre as **primitivas seguras de filesystem e path safety** do DARE CLI nativo em Rust. Comandos futuros leem/escrevem `dare.config.json`, `.dare/**` e `DARE/**`; sem um jail de projeto, path traversal, symlinks e writes parciais podem corromper o repo ou escapar do root.

A entrega são APIs em `dare-core` (`path`, `fs`): `ProjectRoot`, `SafeRelativePath`, bloqueio de escape, tratamento de symlinks/junctions, escrita atómica, backup/restore, normalização POSIX interna, suporte Windows (drive letters/UNC) e file locks. Quem usa são engenheiros dos microplanos 006+; o usuário final ganha operações de disco que falham de forma explícita em escape e não destroem o ficheiro anterior em crash a meio da escrita.

---

## 2. OBJETIVOS E MÉTRICAS DE SUCESSO

| # | Objetivo | Métrica verificável | Meta |
|---|----------|---------------------|------|
| O-01 | Jail de projeto | Qualquer path com `..` / absoluto fora do root → erro tipado | 100% dos casos de escape em suite |
| O-02 | Symlink/junction seguro | Escape via link rejeitado (ou política documentada) | Testes Unix + Windows (quando disponível) |
| O-03 | Escrita atómica | Interromper após write temp não corrompe destino final | Teste de “crash mid-write” simulado |
| O-04 | Backup/restore | Round-trip de ficheiro sob root | Exit 0; conteúdo idêntico |
| O-05 | Paths internos POSIX | Representação canónica usa `/` | Asserts em API de display/serialize |
| O-06 | Windows drive/UNC | Paths com `C:\` e UNC classificados (aceites só se dentro do root) | Testes `cfg(windows)` ou fixtures documentadas |
| O-07 | File locks | Lock exclusivo impede segundo writer no mesmo path | Teste de contenção |
| O-08 | Desbloquear microplano 006 | Checklist MUST do 005 fechado | 100% MUST |

---

## 3. STAKEHOLDERS

| Papel | Nome / Time | Interesse principal |
|-------|-------------|---------------------|
| Product Owner | DARE Labs / Dewtech | Integridade do projeto do usuário |
| Tech Lead | Time DARE CLI Rust | Política symlink, API `ProjectRoot` |
| Engenheiro CLI | Time implementação | Primitivas reutilizáveis para 006+ |
| Usuário Final | Devs / agentes | Ops de disco sem corrupção / escape |
| Segurança | Tech Lead + AppSec | Path traversal, symlink escape (OWASP A01/A03) |
| Operações / CI | CI 003 matrix | Testes Win+Unix |

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Tipo `ProjectRoot` | MUST | Constrói a partir de dir absoluto existente; guarda root canónico; API para resolver paths relativos |
| RF-02 | Tipo `SafeRelativePath` | MUST | Só aceita relativo sem `..` components; rejeita absoluto, vazio perigoso, NUL |
| RF-03 | Resolução segura `root.join(rel) → AbsolutePath` | MUST | Resultado canónico permanece **dentro** do root (prefix check pós-canonicalize quando aplicável) |
| RF-04 | Bloquear path traversal | MUST | Inputs `../`, `foo/../../etc`, encoded variants óbvios → `CoreError` kind `InvalidInput` ou `Io` documentado; mensagem en-US estável |
| RF-05 | Política de symlinks / junctions | MUST | Política fechada no Blueprint (ex.: **rejeitar** symlink cujo target final sai do root; junctions Windows tratados como links) |
| RF-06 | Escrita atómica | MUST | Write em ficheiro temp no mesmo dir → fsync (quando API permitir) → rename; falha antes do rename deixa original intacto |
| RF-07 | Backup | MUST | `backup(path) → backup_path` sob root (ex. `.dare/backup-…` ou sibling `.bak` — **fechar no Blueprint**); não escapa root |
| RF-08 | Restore | MUST | Restaura a partir de backup válido; falha se backup ausente/inválido |
| RF-09 | Normalização POSIX interna | MUST | API `to_posix(&Path) -> String` com `/`; não altera semântica no Windows além da representação |
| RF-10 | Drive letters e UNC (Windows) | MUST | Classificar e rejeitar UNC/drive **fora** do project root; aceitar só se resolve para dentro do root |
| RF-11 | File locks | MUST | Lock exclusivo por path (crate/`std` — **escolher no Blueprint**); segundo acquire falha ou bloqueia com timeout documentado |
| RF-12 | Integração com erros 004 | MUST | Escapes → kinds/exit codes alinhados (tipicamente `InvalidInput`=4 ou `Io`=5); mensagens passam por `redact` |
| RF-13 | Documentação | MUST | `docs/compatibility/path-safety.md` com política symlink, exemplos de rejeição, API overview |
| RF-14 | DEC no decision log | SHOULD | DEC-006 (política symlink + atomic write + lock crate) |
| RF-15 | Paridade golden TS | SHOULD | Mensagem de escape alinhada ou classificada vs baseline (`path must be relative…`) |
| RF-16 | Soft-delete / versioning avançado de backup | COULD | Fora se não necessário aos contratos `.dare`/DARE neste ciclo |

> Prioridades: **MUST** · **SHOULD** · **COULD**

### Contratos de disco afetados (não alterar schema neste ciclo)

| Path | Uso |
|------|-----|
| `dare.config.json` | Futuro loader (008) — só preparar APIs |
| `.dare/**` | Estado/backups |
| `DARE/**` | Artefatos metodologia |

Alteração de schema/ID/exit code público ⇒ ADR + migration (fora do 005 salvo tipagem de erro já coberta pelo 004).

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Segurança | Toda I/O de projeto passa pelas primitivas (convenção + reviews) | 0 APIs públicas que aceitem `PathBuf` absoluto arbitrário sem root |
| RNF-02 | Performance | Canonicalize/lock só no hot path necessário | Ops típicas < 50 ms em SSD local (orientativo) |
| RNF-03 | Compatibilidade | Windows + Unix | Suite CI / testes `cfg` |
| RNF-04 | Observabilidade | Erros de path usam `CoreError` + redact | Sem dumps de paths com secrets |
| RNF-05 | Manutenibilidade | Módulo `dare-core/src/fs` + `path.rs` | Clippy limpo; sem `unwrap` em prod |
| RNF-06 | Testabilidade | Tempdirs isolados (`tempfile`) | Sem depender do CWD do developer |

---

## 6. REQUISITOS DE SEGURANÇA

| ID | Requisito | Referência |
|----|-----------|------------|
| RS-01 | Validar todo path de entrada (relativo, sem NUL, sem traversal) antes de I/O | OWASP A03 |
| RS-02 | Não logar conteúdo de ficheiros; paths em erros passam por hygiene/`redact` quando aplicável | OWASP A02 |
| RS-03 | Jail: nenhuma leitura/escrita fora de `ProjectRoot` via APIs deste módulo | OWASP A01 |
| RS-04 | `cargo audit` + `cargo deny` verdes após novas deps | OWASP A06 |
| RS-05 | Sem secrets em fixtures; tempdirs limpos | Supply chain / hygiene |
| RS-06 | Symlink escape = deny (política default proposta) | Path traversal |
| RS-07 | Atomic replace: não truncar destino antes de conteúdo novo estar fsync’d/renomeado | Integridade |
| RS-08 | Locks evitam corrupção por writers concorrentes do próprio CLI | Concorrência |
| RS-09 | Processos (shell) ficam no 006 — aqui não introduzir `Command` com path não validado | Escopo |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Rust | 1.85.0 | pin existente |
| Erros | `thiserror` / `CoreError` (004) | existente |
| Paths UTF-8 | `camino` (proposta Documento Mestre) | pin no Blueprint |
| Temp / atomic | `tempfile` + rename | pin no Blueprint |
| Locks | `fs2` / `file-lock` / `std` — **A confirmar** | Blueprint |
| Testes | `tempfile`, `assert_fs` (opcional) | pin |
| Walk/glob | **fora** deste ciclo (exceto se necessário a testes) | 006/posteriores |

---

## 8. INTEGRAÇÕES EXTERNAS

| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |
|---------|------|-----------|---------|----------------|-------------|
| Filesystem local | I/O | OS APIs | Entrada+saída | Ficheiros do projeto | Time CLI |
| CI runners Win/Unix | Test | GHA | Entrada | Execução de testes | Time CLI |
| Baseline TS 3.18.1 | Referência | fixtures | Entrada | Mensagens path-safety | Compat |

---

## 9. RESTRIÇÕES

- **Pré-requisitos:** Microplanos **002** e **004** DONE.
- **Prazo:** Bloqueia 006 (processos seguros) e loaders de config posteriores.
- **Limitações:**
  - Não implementar `dare config` / discover / update completos.
  - Não alterar schema de `dare.config.json` neste ciclo.
  - Não execução de subprocessos (006).
  - Não SQLite/GraphRAG disk layout (040+).
  - Política symlink deve ser **uma** e documentada (sem “às vezes segue”).
- **Idioma:** mensagens en-US; docs pt-BR.
- **Breaking:** mudar semântica de escape/exit ⇒ processo ADR.

---

## 10. FORA DO ESCOPO (v1)

- Microplano 006+ (processos, config migrations, comandos).
- Watchers de filesystem / inotify.
- Encriptação at-rest de backups.
- ACLs / permissões Unix avançadas além do necessário ao create/write.
- Rede / remote FS específicos (NFS quirks) além do que os testes CI cobrirem.
- Fuzzing completo (SHOULD futuro) — unit+integration bastam no 005.

---

## 11. RISCOS E MITIGAÇÕES

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|-------|---------------|---------|-----------|
| R-01 | Symlink policy diverge do TS | Média | Alto | DEC + RF-15; documentar Classe |
| R-02 | `canonicalize` falha se path ainda não existe | Alta | Médio | Resolver parent existente + join; documentar |
| R-03 | fsync não disponível / lento em alguns FS | Média | Médio | Best-effort fsync; ainda assim rename atómico |
| R-04 | Locks flaky no Windows | Média | Médio | Crate madura; timeouts; testes `cfg(windows)` |
| R-05 | UNC edge cases | Média | Médio | Rejeitar UNC fora do root por default |
| R-06 | TOCTOU entre check e open | Média | Alto | Open com path já validado; re-check após canonicalize quando possível |

---

## 12. CHECKLIST DE APROVAÇÃO

- [ ] RF-01…RF-16 priorizados
- [ ] Política de symlink (deny escape) aceite ou marcada para Blueprint
- [ ] Local de backup (`.dare/…` vs sibling) a fechar no Blueprint
- [ ] Crate de file lock a escolher no Blueprint
- [ ] RS-01…RS-09 validados
- [ ] Fora de escopo alinhado (sem 006)
- [ ] Pré-requisitos 002+004 confirmados
- [ ] Pronto para `/dare-blueprint` → `DARE/BLUEPRINT-005-filesystem-seguro-e-path-safety.md`

---

## Apêndice A — Crates / paths (microplano)

| Path | Papel |
|------|-------|
| `crates/dare-core/src/path.rs` | `ProjectRoot`, `SafeRelativePath`, normalização POSIX |
| `crates/dare-core/src/fs/` | atomic write, backup/restore, locks |

## Apêndice B — Mensagem de erro (proposta)

Alinhamento SHOULD com baseline TS:

```text
Error: path must be relative and stay within the project
```

(ou equivalente en-US estável via `CoreError::invalid_input` / kind dedicado — Blueprint fecha o kind exato.)

## Apêndice C — Próximas etapas

1. Revisar e aprovar este Design.  
2. `/dare-blueprint` → `DARE/BLUEPRINT-005-filesystem-seguro-e-path-safety.md`.  
3. Após closeout → [`006-execucao-segura-de-processos.md`](../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/006-execucao-segura-de-processos.md).
