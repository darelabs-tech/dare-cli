# DARE CLI

> **Design. Architect. Review. Execute.**  
> Uma metodologia estruturada para desenvolvimento de software assistido por IA, com checkpoints humanos obrigatórios.

---

## O que é o DARE CLI?

O **DARE CLI** é uma ferramenta de linha de comando escrita em Rust que implementa o **Método DARE** — uma metodologia que equilibra a velocidade da IA com estrutura, contexto e revisões humanas em cada transição de fase.

O desenvolvimento de software com IA opera hoje em dois extremos:

| Vibe Coding | Tradicional |
|---|---|
| "Me dá um código que faça X" + esperança | Especificação detalhada feita só por humanos |
| Rápido para protótipo, **caos para evoluir** | Lento, **aproveita pouco a IA** |
| Sem auditabilidade do raciocínio | Sem ganho de produtividade real |

**DARE preenche o gap entre os dois.** Mantém a velocidade da IA, mas com **estrutura, contexto e checkpoints humanos**.

---

## v4.0.0 — 100% Native Rust

A versão atual do DARE CLI foi **completamente reescrita em Rust**:

- ⚡ Execução ultrarrápida — sem Node.js ou npm
- 🦀 Binário único, zero dependências de runtime
- 🗄️ Engine GraphRAG em SQLite/FTS5 nativa
- 🔀 DAG Task Runner com algoritmo de Kahn (paralelismo)
- 🔌 MCP Server embutido como transporte separado
- 🌳 AST parser nativo para engenharia reversa

---

## As 4 Fases do Método

```
1. DESIGN  →  2. ARCHITECT  →  3. REVIEW  →  4. EXECUTE
──────────    ────────────────  ──────────    ──────────────
Humano        IA propõe         Humano        IA implementa
define        arquitetura       valida        + Ralph Loop
requisitos                      e aprova
↓ DESIGN.md   ↓ BLUEPRINT.md    ↓ ✓ approval  ↓ Code + Tests ✓
```

> 💡 **Princípio central:** humanos pensam estratégia (fases 1 e 3), IA executa tática (fases 2 e 4). Cada transição entre fases passa por checkpoint explícito.

---

## Instalação rápida

**macOS, Linux e FreeBSD:**
```bash
curl -fsSL https://darelabs.tech/install | sh
```

**Windows PowerShell:**
```powershell
irm https://darelabs.tech/install.ps1 | iex
```

Siga para [Instalação](getting-started/install.md) para guias detalhados (Homebrew, WinGet, etc.) ou para o [Quickstart](getting-started/quickstart.md) para seu primeiro projeto DARE.

---

## Compatibilidade com Agentes

O DARE CLI expõe todas as suas funcionalidades como **skills nativas** para os principais agentes de IA:

- [Antigravity / Gemini CLI](agents/antigravity.md)
- [Cursor](agents/cursor.md)
- [Claude Code](agents/claude-code.md)
- [Codex (OpenAI)](agents/codex.md)
