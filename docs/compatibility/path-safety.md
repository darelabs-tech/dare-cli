# Path safety (Ciclo 005)

Primitivas de filesystem do DARE CLI nativo em `dare-core` (`path`, `fs`).

## Política

| Tema | Decisão |
|------|---------|
| Jail | Toda I/O exige `ProjectRoot` + `SafeRelativePath` |
| Traversal / absoluto / UNC / drive | `InvalidInput` — `path must be relative and stay within the project` |
| Symlink/junction | Deny se o target final (canonicalize) sai do root |
| Escrita | Atómica: temp no mesmo dir → fsync best-effort → rename |
| Backup | `.dare/backups/<YYYYMMDDThhmmssZ>-<sha8>/<posix-rel>` |
| Lock | `fs4` exclusive `try_lock` em `<file>.darelock` |

## API (overview)

- `SafeRelativePath::new`, `ProjectRoot::new` / `resolve` / `contains`
- `dare_core::fs::read_to_string`, `atomic_write`, `backup`, `restore`, `FileLock::try_acquire`
- `to_posix` — separadores `/`

## Security (RS-01…RS-09)

| ID | Status |
|----|--------|
| RS-01 Validação de path | ✅ |
| RS-02 Sem log de conteúdo; redact em erros | ✅ |
| RS-03 Jail ProjectRoot | ✅ |
| RS-04 audit/deny | ✅ (gate) |
| RS-05 fixtures tempdir | ✅ |
| RS-06 symlink escape deny | ✅ (Unix cfg; lexical Win) |
| RS-07 atomic replace | ✅ |
| RS-08 file locks | ✅ |
| RS-09 sem Command | ✅ |

## Release notes — Ciclo 005

- Path jail + atomic write + backup/restore + file locks
- Pins: `camino 1.1.9`, `tempfile 3.20.0`, `fs4 1.1.0`, `sha2 0.10.9`
- Ver DEC-006

## Referências

- `DARE/DESIGN-005-…`, `DARE/BLUEPRINT-005-…`
- DEC-006 em [`DECISION-LOG.md`](../DECISION-LOG.md)
