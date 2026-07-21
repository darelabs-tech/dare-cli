#!/usr/bin/env python3
"""
Orquestrador do ciclo DARE por microplano — SEM Cursor API key.

O método usa slash commands da sessão da IDE (/dare-design, /dare-blueprint,
/dare-tasks, /dare-dag-run-parallel). Este script NÃO chama Cursor SDK.

Papéis:
  - Script  → descobre microplanos, valida artefatos, avança fases, commit/push
  - Agente  → executa cada slash command com o prompt emitido por --next

Uso (microplanos 011 → 056 até o fim) — dentro do Cursor / Claude Code:

  python scripts/run-microplanos-loop.py init
  python scripts/run-microplanos-loop.py init --start 11 --end 56
  python scripts/run-microplanos-loop.py next
  # …agente executa o slash command…
  python scripts/run-microplanos-loop.py complete
  # repetir até status = done
  python scripts/run-microplanos-loop.py status

Opções:
  --start N      primeiro microplano (default: 8)
  --end N        último microplano inclusive (default: 56); omitir = até o último ficheiro
  --count N      limite opcional (sobrescreve --end se ambos forem passados via slice)
  --only 11,12   lista CSV explícita
  --no-push      no commit final de cada microplano, não faz push
  --dry-run      complete/commit não alteram git
  --no-cleanup   não remove target/ nem worktrees após cada ciclo

Após cada microplano (fase commit), limpa disco:
  - pastas `target/` no repo (artefactos Cargo)
  - `%TEMP%/cursor-sandbox-cache` (e via CARGO_TARGET_DIR)
  - git worktrees extras + `git worktree prune`
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MICROPLANOS_DIR = (
    REPO_ROOT / "DARE-RUST-MICRO-PLANOS" / "DARE-RUST-MICRO-PLANOS"
)
DARE_DIR = REPO_ROOT / "DARE"
STATE_PATH = DARE_DIR / ".microplano-loop-state.json"
NEXT_PROMPT_PATH = DARE_DIR / ".microplano-loop-next.md"

MICROPLANO_RE = re.compile(r"^(\d{3})-(.+)\.md$", re.IGNORECASE)
SKIP_PREFIXES = {"000", "000A", "999"}

# Ordem das fases = slash commands do método DARE
PHASES = ("design", "blueprint", "tasks", "execute", "commit")

SLASH = {
    "design": "/dare-design",
    "blueprint": "/dare-blueprint",
    "tasks": "/dare-tasks",
    "execute": "/dare-dag-run-parallel",
    "commit": None,  # script faz git
}


@dataclass
class Microplano:
    number: int
    slug: str
    path: str

    @property
    def nnn(self) -> str:
        return f"{self.number:03d}"

    @property
    def label(self) -> str:
        return f"{self.nnn}-{self.slug}"

    def design(self) -> Path:
        return DARE_DIR / f"DESIGN-{self.nnn}-{self.slug}.md"

    def blueprint(self) -> Path:
        return DARE_DIR / f"BLUEPRINT-{self.nnn}-{self.slug}.md"

    def tasks(self) -> Path:
        return DARE_DIR / f"TASKS-{self.nnn}-{self.slug}.md"

    def dag(self) -> Path:
        return DARE_DIR / f"dare-dag-{self.nnn}.yaml"

    def execution(self) -> Path:
        return DARE_DIR / f"EXECUTION-{self.nnn}"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def discover(directory: Path, start: int) -> list[Microplano]:
    found: list[Microplano] = []
    for path in sorted(directory.glob("*.md")):
        m = MICROPLANO_RE.match(path.name)
        if not m:
            continue
        nnn, slug = m.group(1), m.group(2)
        if nnn in SKIP_PREFIXES or not nnn.isdigit():
            continue
        number = int(nnn)
        if number < start:
            continue
        found.append(Microplano(number=number, slug=slug, path=str(path.resolve())))
    return sorted(found, key=lambda x: x.number)


def load_state() -> dict[str, Any]:
    if not STATE_PATH.exists():
        print(
            f"ERRO: estado não encontrado. Rode: python scripts/run-microplanos-loop.py init",
            file=sys.stderr,
        )
        sys.exit(1)
    return json.loads(STATE_PATH.read_text(encoding="utf-8"))


def save_state(state: dict[str, Any]) -> None:
    DARE_DIR.mkdir(parents=True, exist_ok=True)
    state["updated_at"] = now_iso()
    STATE_PATH.write_text(
        json.dumps(state, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def current_mp(state: dict[str, Any]) -> Microplano | None:
    idx = state["index"]
    queue = state["queue"]
    if idx >= len(queue):
        return None
    raw = queue[idx]
    return Microplano(**raw)


_SPEC_REQUIRED = re.compile(
    r"(?i)(##\s*OBJETIVO|##\s*Objetivo|##\s*VALIDATION|##\s*Validation|"
    r"##\s*DONE|##\s*Definition of Done|##\s*Arquivos)"
)


def _execution_specs_ok(mp: Microplano) -> tuple[bool, str]:
    """Rejeita stubs (# id apenas). Cada spec precisa de corpo DARE real."""
    ex = mp.execution()
    if not ex.is_dir():
        return False, f"{ex}/ (ausente)"
    specs = sorted(ex.glob("*.md"))
    if not specs:
        return False, f"{ex}/ (sem *.md)"
    stubs: list[str] = []
    for p in specs:
        try:
            text = p.read_text(encoding="utf-8")
        except OSError as e:
            return False, f"{p}: {e}"
        # Stub típico do atalho errado: só "# mp014-001" (+ newlines)
        if p.stat().st_size < 180 or not _SPEC_REQUIRED.search(text):
            stubs.append(p.name)
    if stubs:
        return False, (
            f"{ex}/ stubs sem spec DARE ({len(stubs)}): "
            + ", ".join(stubs[:8])
            + ("…" if len(stubs) > 8 else "")
            + " — use /dare-tasks (OBJETIVO + VALIDATION + DONE)"
        )
    return True, f"{ex}/ ({len(specs)} specs)"


def artifact_ok(mp: Microplano, phase: str) -> tuple[bool, str]:
    if phase == "design":
        p = mp.design()
        return p.is_file() and p.stat().st_size > 200, str(p)
    if phase == "blueprint":
        p = mp.blueprint()
        return p.is_file() and p.stat().st_size > 200, str(p)
    if phase == "tasks":
        if not (mp.tasks().is_file() and mp.dag().is_file()):
            return False, f"{mp.tasks()} + {mp.dag()} + {mp.execution()}/"
        specs_ok, specs_detail = _execution_specs_ok(mp)
        ok = specs_ok
        return ok, f"{mp.tasks()} + {mp.dag()} + {specs_detail}"
    if phase == "execute":
        # Specs ainda têm de ser reais (ninguém pode apagar/esvaziar na execute)
        specs_ok, specs_detail = _execution_specs_ok(mp)
        if not specs_ok:
            return False, specs_detail
        tasks = mp.tasks()
        if not tasks.is_file():
            return False, str(tasks)
        text = tasks.read_text(encoding="utf-8")
        pending = len(re.findall(r"PENDING|⏳", text))
        done = len(re.findall(r"DONE|✅", text))
        # Aceita se não houver PENDING e houver pelo menos 1 DONE
        ok = pending == 0 and done > 0
        return ok, f"{tasks} (DONE≈{done}, PENDING≈{pending}); {specs_detail}"
    if phase == "commit":
        return True, "git"
    return False, "fase desconhecida"


def build_slash_prompt(mp: Microplano, phase: str) -> str:
    slash = SLASH[phase]
    if phase == "design":
        return f"""# Próximo passo DARE (slash command)

Execute **{slash}** + o arquivo do Microplano.

## Microplano
`{mp.path}`

## Saída canônica obrigatória
`{mp.design().as_posix()}`

## Regras
- Siga a skill/command {slash} (template DESIGN completo).
- MODO AUTONOMO: nao pergunte nada ao humano; nao peca aprovacao; nao espere confirmacao.
- Pode sobrescrever o DESIGN deste microplano ({mp.nnn}) se estiver a repetir o ciclo.
- Nao sobrescreva designs de microplanos anteriores (001–{mp.number - 1:03d}).
- Espelhe o padrao de `DARE/DESIGN-007-contratos-persistidos.md`.
- Stack = este repo Rust (workspace dare-cli).
- Documentacao DARE em portugues.
- PROIBIDO pular para implementacao nesta fase.

Ao terminar a fase, IMEDIATAMENTE rode:
```
python scripts/run-microplanos-loop.py complete
```
e continue a proxima fase sem pausar.
"""

    if phase == "blueprint":
        return f"""# Próximo passo DARE (slash command)

Execute **{slash}** + o arquivo de design gerado.

## Design
`{mp.design().as_posix()}`

## Saída canônica obrigatória
`{mp.blueprint().as_posix()}`

## Regras
- Siga {slash}: gere SOMENTE o BLUEPRINT (sem TASKS/DAG/EXECUTION).
- Anti-stub contract completo.
- Padrão: `DARE/BLUEPRINT-007-contratos-persistidos.md`.
- MODO AUTONOMO: nao pergunte nada; nao peca aprovacao.

Ao terminar, IMEDIATAMENTE:
```
python scripts/run-microplanos-loop.py complete
```
e siga a proxima fase.
"""

    if phase == "tasks":
        return f"""# Proximo passo DARE (slash command)

Execute **{slash}** (+ `/dare-tasks`) + o blueprint gerado.

## Blueprint
`{mp.blueprint().as_posix()}`

## Saidas canonicas
- `{mp.tasks().as_posix()}`
- `{mp.dag().as_posix()}`
- `{mp.execution().as_posix()}/` com specs `mp{mp.nnn}-*.md`

## Regras (INEGOCIAVEIS)
- IDs `mp{mp.nnn}-001`, ...
- Consistencia TASKS <-> DAG <-> EXECUTION (dare-dag-runner).
- Padrao: `DARE/EXECUTION-008/mp008-*.md` (OBJETIVO + VALIDATION/DONE + caminhos).
- PROIBIDO criar `EXECUTION-*/mp*.md` stub (`# mp014-001` sozinho). Spec vazia = fase tasks INVALIDA.
- Anti-stub: cada spec com arquivos exatos, assinaturas, testes nomeados, Definition of Done.
- Nesta fase NAO implemente codigo de producao — so artefatos TASKS/DAG/EXECUTION.
- MODO AUTONOMO: nao pergunte nada; nao peca aprovacao.

Ao terminar, IMEDIATAMENTE:
```
python scripts/run-microplanos-loop.py complete
```
e siga a proxima fase.
"""

    if phase == "execute":
        return f"""# Proximo passo DARE (slash command)

Execute **{slash}** + o DAG gerado.

## DAG
`{mp.dag().as_posix()}`

## Specs (fonte da verdade — LEIA ANTES DE CODIGAR)
`{mp.execution().as_posix()}/mp{mp.nnn}-*.md`

## Comandos do orquestrador CLI
```
dare execute --dag {mp.dag().as_posix()} --status
dare execute --dag {mp.dag().as_posix()} --next
dare execute --dag {mp.dag().as_posix()} --complete <id> --output "..."
dare execute --dag {mp.dag().as_posix()} --fail <id> --reason "..."
```

## Regras (INEGOCIAVEIS)
- Ordem: ler spec EXECUTION da task → implementar exatamente o que a spec pede → Ralph Loop → so entao marcar DONE.
- PROIBIDO inventar implementacao sem spec; PROIBIDO marcar DONE com EXECUTION stub.
- Siga a skill {slash} (fan-out paralelo se possivel; senao serial).
- Ralph Loop em cada task (build -> test -> lint -> audit se deps).
- Atualize `{mp.tasks().as_posix()}` para 100% DONE.
- NAO faca git commit/push (o script fara na fase commit).
- Canvas: `DARE/.canvas.md`.
- MODO AUTONOMO: nao pergunte nada; nao peca aprovacao entre tasks.

Quando todas as tasks estiverem DONE, IMEDIATAMENTE:
```
python scripts/run-microplanos-loop.py complete
```
e continue.
"""

    return "# fase commit — o script executa sozinho"


def write_next_prompt(mp: Microplano, phase: str) -> Path:
    body = build_slash_prompt(mp, phase)
    NEXT_PROMPT_PATH.write_text(body, encoding="utf-8")
    return NEXT_PROMPT_PATH


def run_git(args: list[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=check,
    )


def _dir_size_mb(path: Path) -> float:
    total = 0
    try:
        for p in path.rglob("*"):
            if p.is_file():
                try:
                    total += p.stat().st_size
                except OSError:
                    pass
    except OSError:
        return 0.0
    return total / (1024 * 1024)


def find_target_dirs() -> list[Path]:
    """Pastas `target/` sob o repo + CARGO_TARGET_DIR relacionado."""
    found: list[Path] = []
    seen: set[Path] = set()

    def add(path: Path) -> None:
        try:
            resolved = path.resolve()
        except OSError:
            return
        if resolved in seen or not path.is_dir():
            return
        seen.add(resolved)
        found.append(path)

    add(REPO_ROOT / "target")

    git_dir = (REPO_ROOT / ".git").resolve()
    for p in REPO_ROOT.rglob("target"):
        if p.name != "target" or not p.is_dir():
            continue
        try:
            if git_dir in p.resolve().parents or p.resolve() == git_dir:
                continue
        except OSError:
            continue
        add(p)

    cargo_td = os.environ.get("CARGO_TARGET_DIR", "").strip()
    if cargo_td:
        td = Path(cargo_td)
        if td.is_dir():
            try:
                td.resolve().relative_to(REPO_ROOT.resolve())
                add(td)
            except ValueError:
                lowered = str(td).lower()
                if "dare-cli" in lowered or "cargo-target" in lowered:
                    add(td)

    return found


def _cursor_sandbox_cache_roots() -> list[Path]:
    """Localiza a raiz `cursor-sandbox-cache` (Temp + derivado de CARGO_TARGET_DIR)."""
    candidates: list[Path] = []
    seen: set[Path] = set()

    def add_root(path: Path) -> None:
        try:
            resolved = path.resolve()
        except OSError:
            return
        if resolved in seen:
            return
        if path.is_dir() and path.name.lower() == "cursor-sandbox-cache":
            seen.add(resolved)
            candidates.append(path)

    temp = os.environ.get("TEMP") or os.environ.get("TMP") or ""
    local_app = os.environ.get("LOCALAPPDATA") or ""
    for base in filter(None, [temp, local_app and str(Path(local_app) / "Temp")]):
        add_root(Path(base) / "cursor-sandbox-cache")

    cargo_td = os.environ.get("CARGO_TARGET_DIR", "").strip()
    if cargo_td:
        cur = Path(cargo_td)
        # sobe até encontrar o diretório chamado cursor-sandbox-cache
        for parent in [cur, *cur.parents]:
            if parent.name.lower() == "cursor-sandbox-cache":
                add_root(parent)
                break

    return candidates


def cleanup_cursor_sandbox_cache(dry_run: bool) -> None:
    """Remove o cache Cursor sandbox (costuma ser o maior consumidor de HD)."""
    roots = _cursor_sandbox_cache_roots()
    if not roots:
        print("[cleanup] nenhum cursor-sandbox-cache encontrado")
        return

    for root in roots:
        size = _dir_size_mb(root)
        print(f"[cleanup] removendo cursor-sandbox-cache (~{size:.1f} MiB): {root}")
        if dry_run:
            continue
        try:
            shutil.rmtree(root, ignore_errors=False)
            print(f"[cleanup] removido: {root}")
        except OSError as e:
            print(f"[cleanup] falha ao remover {root}: {e}", file=sys.stderr)
            shutil.rmtree(root, ignore_errors=True)
            if not root.exists():
                print(f"[cleanup] removido (best-effort): {root}")
            else:
                # tenta limpar conteúdo se a raiz estiver bloqueada
                for child in list(root.iterdir()) if root.is_dir() else []:
                    if child.is_dir():
                        shutil.rmtree(child, ignore_errors=True)
                    else:
                        try:
                            child.unlink()
                        except OSError:
                            pass
                print(f"[cleanup] conteúdo limpo best-effort em: {root}")


def cleanup_worktrees(dry_run: bool) -> None:
    """Remove worktrees extras (não o main) e faz prune."""
    listed = run_git(["worktree", "list", "--porcelain"], check=False)
    if listed.returncode != 0:
        print(f"[cleanup] git worktree list falhou: {listed.stderr.strip()}")
        return

    main_path = REPO_ROOT.resolve()
    worktrees: list[Path] = []
    for line in listed.stdout.splitlines():
        if line.startswith("worktree "):
            worktrees.append(Path(line[len("worktree ") :]).resolve())

    extras = [w for w in worktrees if w != main_path]
    if not extras:
        print("[cleanup] nenhum git worktree extra")
    else:
        for wt in extras:
            print(f"[cleanup] removendo worktree: {wt}")
            if dry_run:
                continue
            rm = run_git(["worktree", "remove", "--force", str(wt)], check=False)
            if rm.returncode != 0:
                print(
                    f"[cleanup] worktree remove avisou: "
                    f"{rm.stderr.strip() or rm.stdout.strip()}"
                )
                if wt.exists():
                    shutil.rmtree(wt, ignore_errors=True)

    if dry_run:
        print("[dry-run] git worktree prune")
        return
    pruned = run_git(["worktree", "prune", "-v"], check=False)
    if pruned.stdout.strip():
        print(f"[cleanup] prune:\n{pruned.stdout.strip()}")
    else:
        print("[cleanup] git worktree prune ok")


def cleanup_disk(dry_run: bool = False) -> None:
    """Liberta HD após um ciclo: target/ + cursor-sandbox-cache + worktrees."""
    print("[cleanup] a limpar artefactos de disco do ciclo…")

    targets = find_target_dirs()
    if not targets:
        print("[cleanup] nenhuma pasta target/ encontrada")
    for t in targets:
        size = _dir_size_mb(t)
        print(f"[cleanup] removendo target/ (~{size:.1f} MiB): {t}")
        if dry_run:
            continue
        try:
            shutil.rmtree(t, ignore_errors=False)
            print(f"[cleanup] removido: {t}")
        except OSError as e:
            print(f"[cleanup] falha ao remover {t}: {e}", file=sys.stderr)
            shutil.rmtree(t, ignore_errors=True)

    cleanup_cursor_sandbox_cache(dry_run=dry_run)
    cleanup_worktrees(dry_run=dry_run)
    print("[cleanup] ciclo de limpeza concluído")


def do_commit_push(mp: Microplano, push: bool, dry_run: bool) -> None:
    msg = f"feat(mp{mp.nnn}): concluir microplano {mp.label}"
    st = run_git(["status", "--porcelain"], check=False)
    if not st.stdout.strip():
        print(f"[git] working tree limpa — nada a commitar ({mp.label})")
        return
    print(st.stdout)
    if dry_run:
        print(f"[dry-run] commit: {msg}")
        if push:
            print("[dry-run] push")
        return
    run_git(["add", "-A"])
    run_git(["commit", "-m", msg])
    print(f"[git] commit OK: {msg}")
    if push:
        run_git(["push", "-u", "origin", "HEAD"])
        print("[git] push OK")
    else:
        print("[git] push pulado (--no-push)")


def cmd_init(args: argparse.Namespace) -> None:
    directory = args.microplanos_dir
    if not directory.is_dir():
        print(f"ERRO: pasta não encontrada: {directory}", file=sys.stderr)
        sys.exit(1)

    all_mps = discover(directory, start=0)
    if args.only.strip():
        wanted = {int(x.strip()) for x in args.only.split(",") if x.strip()}
        selected = [m for m in all_mps if m.number in wanted]
    else:
        start = args.start
        end = args.end
        if end == 0:
            end = None
        selected = [m for m in all_mps if m.number >= start]
        if end is not None:
            selected = [m for m in selected if m.number <= end]
        if args.count is not None:
            selected = selected[: args.count]

    if not selected:
        print("Nenhum microplano selecionado.", file=sys.stderr)
        sys.exit(1)

    state = {
        "version": 1,
        "created_at": now_iso(),
        "updated_at": now_iso(),
        "microplanos_dir": str(directory.resolve()),
        "push": not args.no_push,
        "dry_run": bool(args.dry_run),
        "cleanup": not args.no_cleanup,
        "range": {
            "start": selected[0].number,
            "end": selected[-1].number,
            "total": len(selected),
        },
        "index": 0,
        "phase": "design",
        "queue": [asdict(m) for m in selected],
        "history": [],
        "method": "slash-commands-ide-session",
        "note": "Sem CURSOR_API_KEY. Agente da IDE executa /dare-* ; este script só orquestra.",
    }
    save_state(state)
    mp = selected[0]
    prompt_path = write_next_prompt(mp, "design")

    print("Loop inicializado (slash commands / sessao IDE — sem API key).")
    print(f"Estado: {STATE_PATH}")
    print(
        f"Fila ({len(selected)}): {selected[0].label} … {selected[-1].label}"
    )
    if len(selected) <= 12:
        print(f"  ids: {[m.label for m in selected]}")
    else:
        print(f"  ids: {[m.label for m in selected[:4]]} … {[m.label for m in selected[-3:]]}")
    print(f"Fase atual: design -> {SLASH['design']}")
    print(f"Prompt: {prompt_path}")
    print()
    print("MODO AUTONOMO: o agente deve executar o slash em .microplano-loop-next.md,")
    print("rodar `complete`, e repetir sem perguntar ao humano ate status=done.")


def cmd_status(_: argparse.Namespace) -> None:
    state = load_state()
    mp = current_mp(state)
    print(f"index={state['index']}/{len(state['queue'])} phase={state['phase']}")
    rng = state.get("range") or {}
    if rng:
        print(f"range={rng.get('start')}..{rng.get('end')} total={rng.get('total')}")
    print(f"push={state.get('push')} dry_run={state.get('dry_run')} cleanup={state.get('cleanup', True)}")
    if mp is None:
        print("STATUS: done — fila vazia")
        return
    print(f"microplano={mp.label}")
    print(f"slash={SLASH.get(state['phase'])}")
    for phase in PHASES:
        ok, detail = artifact_ok(mp, phase)
        mark = "[OK]" if ok else "[  ]"
        print(f"  {mark} {phase}: {detail}")
    if state.get("history"):
        print("histórico:")
        for h in state["history"][-8:]:
            print(f"  - {h}")


def cmd_next(args: argparse.Namespace) -> None:
    state = load_state()
    mp = current_mp(state)
    if mp is None:
        payload = {"status": "done", "message": "Todos os microplanos da fila concluídos."}
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return

    phase = state["phase"]
    if phase == "commit":
        # commit é do script — complete cuida
        payload = {
            "status": "ready",
            "microplano": mp.label,
            "phase": phase,
            "slash": None,
            "action": "run: python scripts/run-microplanos-loop.py complete",
            "note": "Fase commit é automática no complete.",
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return

    prompt_path = write_next_prompt(mp, phase)
    payload = {
        "status": "ready",
        "microplano": mp.label,
        "phase": phase,
        "slash": SLASH[phase],
        "prompt_file": str(prompt_path),
        "inputs": {
            "microplano": mp.path if phase == "design" else None,
            "design": str(mp.design()) if phase == "blueprint" else None,
            "blueprint": str(mp.blueprint()) if phase == "tasks" else None,
            "dag": str(mp.dag()) if phase == "execute" else None,
        },
        "instruction": (
            f"Leia {prompt_path.name} e execute o slash command "
            f"{SLASH[phase]} conforme o arquivo. Não use CURSOR_API_KEY / cursor-sdk."
        ),
    }
    text = json.dumps(payload, ensure_ascii=False, indent=2)
    print(text)
    if not args.json_only:
        print("\n----- PROMPT -----\n")
        print(prompt_path.read_text(encoding="utf-8"))


def advance(state: dict[str, Any], mp: Microplano, phase: str) -> None:
    state["history"].append(
        {"at": now_iso(), "microplano": mp.label, "phase": phase, "result": "ok"}
    )
    idx = PHASES.index(phase)
    if idx + 1 < len(PHASES):
        state["phase"] = PHASES[idx + 1]
    else:
        # próximo microplano
        state["index"] += 1
        state["phase"] = "design"
    save_state(state)


def cmd_complete(args: argparse.Namespace) -> None:
    state = load_state()
    mp = current_mp(state)
    if mp is None:
        print("Fila já concluída.")
        return

    phase = state["phase"]
    push = state.get("push", True) and not args.no_push
    dry_run = state.get("dry_run", False) or args.dry_run

    if phase == "commit":
        do_commit_push(mp, push=push, dry_run=dry_run)
        do_cleanup = state.get("cleanup", True) and not args.no_cleanup
        if do_cleanup:
            cleanup_disk(dry_run=dry_run)
        else:
            print("[cleanup] pulado (--no-cleanup)")
        advance(state, mp, phase)
        nxt = current_mp(load_state())
        if nxt is None:
            print("STATUS: done — todos os microplanos concluídos.")
            if NEXT_PROMPT_PATH.exists():
                NEXT_PROMPT_PATH.unlink()
            return
        write_next_prompt(nxt, "design")
        print(f"Avancou para {nxt.label} / design -> {SLASH['design']}")
        print(f"Prompt: {NEXT_PROMPT_PATH}")
        return

    ok, detail = artifact_ok(mp, phase)
    if not ok and not args.force:
        print(
            f"ERRO: artefato da fase '{phase}' incompleto: {detail}\n"
            f"Execute o slash {SLASH[phase]} e tente de novo, ou use --force.",
            file=sys.stderr,
        )
        sys.exit(2)

    print(f"[ok] {mp.label} / {phase}: {detail}")
    advance(state, mp, phase)
    state = load_state()
    nxt_mp = current_mp(state)
    if nxt_mp is None:
        print("STATUS: done")
        return

    nxt_phase = state["phase"]
    if nxt_phase == "commit":
        print(f"Proximo: {nxt_mp.label} / commit (rode complete de novo)")
    else:
        write_next_prompt(nxt_mp, nxt_phase)
        print(f"Proximo: {nxt_mp.label} / {nxt_phase} -> {SLASH[nxt_phase]}")
        print(f"Prompt: {NEXT_PROMPT_PATH}")
        print("Agente: leia DARE/.microplano-loop-next.md e execute o slash command.")


def cmd_run_agent_help(_: argparse.Namespace) -> None:
    """Instruções coláveis no chat do Cursor."""
    print(
        """
# Como rodar este loop NO CURSOR (slash commands, sem API key)

1. No terminal do projeto:
   python scripts/run-microplanos-loop.py init
   # default: --start 8 --end 56

2. No chat do agente Cursor, cole:

   Leia DARE/.microplano-loop-next.md e execute exatamente o slash command
   indicado (/dare-design, /dare-blueprint, /dare-tasks ou /dare-dag-run-parallel),
   usando o arquivo de entrada citado. Não use CURSOR_API_KEY nem cursor-sdk.
   Ao terminar a fase, rode: python scripts/run-microplanos-loop.py complete
   Depois leia de novo .microplano-loop-next.md e continue até o status done.
   Após cada microplano a fase commit faz git commit+push via script,
   depois limpa `target/` e git worktrees (use --no-cleanup para pular).

3. Acompanhe: python scripts/run-microplanos-loop.py status
   Limpeza manual: python scripts/run-microplanos-loop.py cleanup
""".strip()
    )


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="Orquestra microplanos DARE via slash commands (sem API key)."
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    def add_common(sp: argparse.ArgumentParser) -> None:
        sp.add_argument("--microplanos-dir", type=Path, default=DEFAULT_MICROPLANOS_DIR)
        sp.add_argument("--no-push", action="store_true")
        sp.add_argument("--dry-run", action="store_true")
        sp.add_argument(
            "--no-cleanup",
            action="store_true",
            help="Nao remove target/ nem worktrees apos cada ciclo",
        )

    sp = sub.add_parser("init", help="Inicializa fila (default: 008–056)")
    add_common(sp)
    sp.add_argument("--start", type=int, default=8, help="Primeiro microplano (default: 8)")
    sp.add_argument(
        "--end",
        nargs="?",
        const=0,
        default=56,
        type=int,
        help=(
            "Ultimo microplano inclusive (default: 56). "
            "Passe --end sem valor (ou --end 0) para ir ate o ultimo ficheiro."
        ),
    )
    sp.add_argument(
        "--count",
        type=int,
        default=None,
        help="Limite opcional de quantidade (apos aplicar start/end)",
    )
    sp.add_argument("--only", default="", help="CSV de números, ex. 11,12,13")
    sp.set_defaults(func=cmd_init)

    sp = sub.add_parser("next", help="Emite a fase/slash atuais")
    sp.add_argument("--json-only", action="store_true")
    sp.set_defaults(func=cmd_next)

    sp = sub.add_parser("complete", help="Valida artefato e avança fase")
    sp.add_argument("--force", action="store_true", help="Avança sem validar artefato")
    sp.add_argument("--no-push", action="store_true")
    sp.add_argument("--dry-run", action="store_true")
    sp.add_argument(
        "--no-cleanup",
        action="store_true",
        help="Nao remove target/ nem worktrees neste complete de commit",
    )
    sp.set_defaults(func=cmd_complete)

    sp = sub.add_parser("status", help="Mostra progresso")
    sp.set_defaults(func=cmd_status)

    sp = sub.add_parser(
        "cleanup",
        help="Remove target/ e worktrees agora (sem avancar fases)",
    )
    sp.add_argument("--dry-run", action="store_true")
    sp.set_defaults(func=lambda a: cleanup_disk(dry_run=a.dry_run))

    sp = sub.add_parser("help-agent", help="Texto para colar no chat do Cursor")
    sp.set_defaults(func=cmd_run_agent_help)

    return p


def main() -> None:
    # Windows cp1252: evita crash em prints UTF-8 do loop
    if hasattr(sys.stdout, "reconfigure"):
        try:
            sys.stdout.reconfigure(encoding="utf-8", errors="replace")
            sys.stderr.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
