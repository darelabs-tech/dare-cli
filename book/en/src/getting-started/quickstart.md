# Quickstart

This guide covers everything you need to create your first project with DARE CLI in under 5 minutes.

---

## Step 1 — Create a new project

```bash
mkdir my-project && cd my-project
dare init
```

The `dare init` command will ask you for:
1. **Project name** (e.g., `my-api`)
2. **Stack** (e.g., `rust`, `python`, `node`, `laravel`, `go`, `rails`)
3. **MCP transport** (optional)

## Step 2 — Bootstrap the stack

Apply the scaffold for the chosen stack:

```bash
dare bootstrap
```

This materializes the folder structure, config files, and harnesses for the AI agents.

## Step 3 — Create the Design

Describe what you want to build:

```bash
dare design "I want a JWT authentication REST API in Rust"
```

This generates `DARE/DESIGN.md` — the requirement document.

## Step 4 — Generate the Blueprint

With the Design approved, the AI proposes the architecture:

```bash
dare blueprint
```

This generates `DARE/BLUEPRINT.md` with layers, endpoints, data models, and a task list.

## Step 5 — Execute Tasks

Start implementing task by task:

```bash
dare execute task-001
```

The **Ralph Loop** runs: it implements code, runs validation gates (tests, clippy, fmt), and auto-corrects until they pass.
