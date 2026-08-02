<div align="center">

![DARE Labs](assets/brand/darelabs-wordmark.png)

# DARE Method CLI

### Design. Architect. Review. Execute.

**A structured methodology for AI-assisted software development with mandatory human-in-the-loop reviews.**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Built by Dare Labs](https://img.shields.io/badge/built%20by-Dare%20Labs-0070f3)](https://darelabs.tech)
[![Docs](https://img.shields.io/badge/docs-Dare_CLI_Book-2563eb?logo=gitbook)](https://darelabs-tech.github.io/dare-cli/)
[![Rust](https://img.shields.io/badge/Rust-1.83+-orange.svg)](https://www.rust-lang.org/)
[![Antigravity](https://img.shields.io/badge/Antigravity-supported-7928ca)](#)
[![Cursor IDE](https://img.shields.io/badge/Cursor-IDE-000000?logo=cursor)](#)

> 🚀 **v4.0.0** — **100% Native Rust CLI**. 
> The DARE CLI has been fully rewritten in Rust. Lightning fast execution, embedded Neo4j/SQLite graph database, native AST parser, Kahn's algorithm DAG runner, and zero Node.js/npm dependencies required. 

[**Quickstart**](https://darelabs-tech.github.io/dare-cli/) ·
[**O Método**](https://darelabs-tech.github.io/dare-cli/) ·
[**Ralph Loop**](https://darelabs-tech.github.io/dare-cli/) ·
[**Comandos**](https://darelabs-tech.github.io/dare-cli/)

<br/>

[![Docs PT-BR](https://img.shields.io/badge/📖%20DOCS-PORTUGUÊS-yellow?style=for-the-badge)](https://dewtech-technologies.github.io/dare-method/pt/)
[![Docs EN](https://img.shields.io/badge/📖%20DOCS-ENGLISH-green?style=for-the-badge)](https://dewtech-technologies.github.io/dare-method/en/)

</div>

---

## ⚡ Quickstart em 1 minuto

**macOS, Linux, and FreeBSD:**
```bash
curl -fsSL https://darelabs.tech/install | sh
```

**Windows PowerShell:**
```powershell
irm https://darelabs.tech/install.ps1 | iex
```

> **Node.js / npm is NO longer required.** DARE v4.0.0+ runs entirely as a compiled binary.

### Seu primeiro projeto

```bash
# 1. Crie uma pasta para o seu projeto e entre nela
mkdir meu-projeto && cd meu-projeto

# 2. Inicialize o DARE
dare init
# → A CLI te guiará pelas opções de stack, GraphRAG e ferramentas.

# 3. Dispare o primeiro comando para a IA
dare design "Quero uma API de autenticação JWT em Rust"
```

---

## 🎯 O Problema

O desenvolvimento de software com IA hoje opera em dois extremos:

| Vibe Coding | Tradicional |
|---|---|
| "Me dá um código que faça X" + esperança | Especificação detalhada feita só por humanos |
| Rápido pra protótipo, **caos pra evoluir** | Lento, **aproveita pouco a IA** |
| Sem auditabilidade do raciocínio | Sem ganho de produtividade real |

**DARE preenche o gap entre os dois.** Mantém a velocidade da IA, mas com **estrutura, contexto e checkpoints humanos**.

---

## 🚀 O Método

DARE é o acrônimo de **4 fases sequenciais** com responsabilidades claras:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   1. DESIGN     →  2. ARCHITECT  →  3. REVIEW   →  4. EXECUTE          │
│   ─────────       ─────────────     ─────────      ─────────            │
│   Humano          IA propõe         Humano         IA implementa       │
│   define          arquitetura       valida         + Ralph Loop        │
│   requisitos                        e aprova                            │
│                                                                         │
│   ↓ DESIGN.md     ↓ BLUEPRINT.md    ↓ ✓ approval   ↓ Code + Tests ✓    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

| Fase | O que faz | Quem faz | Saída |
|------|-----------|----------|-------|
| **1. Design** | Define **o que** vamos construir e **por quê** | Humano (IA auxilia) | `DARE/DESIGN.md` |
| **2. Architect** | Decide **como** vamos construir, em arquitetura e tasks | IA propõe, humano valida | `DARE/BLUEPRINT.md` |
| **3. Review** | Aprova ou ajusta o plano antes de gastar tokens | Humano | ✓ approval explícito |
| **4. Execute** | Implementa task por task, com **Ralph Loop** rodando até gates passarem | IA | Código + testes verdes |

> 💡 **Princípio central:** humanos pensam estratégia (1 e 3), IA executa tática (2 e 4). Cada transição entre fases passa por checkpoint explícito.

---

## 🤡 Ralph Loop

<div align="center">

<img src="assets/brand/ralph-loop.webp" alt="Ralph Wiggum — I'm in danger" width="320"/>

*"I'm in danger 😄"*

</div>

Inspirado no **Ralph Wiggum** dos Simpsons, o **Ralph Loop** é o ciclo de **auto-correção pós-execução** que acontece dentro da fase 4 (Execute).

Agentes de IA são excelentes em iteração até o objetivo, mas ruins em planejamento estratégico. O Ralph Loop usa essa força:

1. IA implementa a task e escreve o código.
2. A CLI roda os **Validation Gates** (testes unitários, linter, formatter, type checker).
3. Se falhar, a IA lê o erro, corrige e tenta de novo, iterando ininterruptamente até a casa parar de pegar fogo (testes passarem).

---

## 🔌 Principais Ferramentas da v4.0.0

A versão em Rust do DARE CLI incorpora nativamente as seguintes engines:

- **DAG Task Runner (`dare dag run`)**: Executa tasks independentes em paralelo (Kahn's algorithm), reduzindo o tempo de execução em até 75%.
- **Engine GraphRAG (`dare graph`)**: Grafo de conhecimento de *requisito ↔ código* totalmente reconstruído em Rust usando SQLite/FTS5 (ou Neo4j opcional). Extração semântica ultrarrápida.
- **MCP Server Embutido (`dare-mcp-server`)**: Economiza até 95% de tokens na execução das tarefas provendo contexto arquitetural via Model Context Protocol sem precisar expor o BLUEPRINT.md completo.
- **Engenharia Reversa Rápida (`dare reverse`)**: Fase 0 para legados. Mapeamento de ast/módulos na velocidade do Rust.

---

## 📋 Comandos Principais

| Comando | Descrição |
|---------|-----------|
| `dare init` | Inicializa a infraestrutura DARE num projeto limpo. |
| `dare discover` | Detecção brownfield. Instala o DARE em projetos existentes sem tocar no código atual. |
| `dare reverse` | Faz a engenharia reversa de projetos legados (Fase 0). |
| `dare design` | Inicia o questionário de design e especificação de produto. |
| `dare blueprint`| Transforma o Design em um Blueprint arquitetural de software. |
| `dare execute` | Entra no Ralph Loop executando tasks individuais (e.g. `dare execute task-001`). |
| `dare review` | Detecta TODOs deixados pela IA, stubs e mocks vazios. |
| `dare update` | Mantém os templates, hooks e skills da CLI sincronizados na pasta `.agents`. |
| `dare info` | Retorna o status de integridade do método e engine. |

> Todas essas funcionalidades também estão expostas para **Antigravity**, **Cursor** e **Claude Code** no formato de skills nativas (e.g. `/dare-design`, `/dare-execute`).

---

## Documentação Completa

| Doc | Propósito |
|-----|---------|
| [`docs/migration/install-rust.md`](docs/migration/install-rust.md) | Guia completo de instalação |
| [`docs/migration/RELEASE-NOTES-stable.md`](docs/migration/RELEASE-NOTES-stable.md) | Release notes da versão Stable (v4.0.0) |
| [`docs/migration/final-compatibility-report.md`](docs/migration/final-compatibility-report.md) | Relatório de Paridade Rust ↔ TypeScript |
| [`docs/compatibility/README.md`](docs/compatibility/README.md) | Specs técnicas de baseline e features paritárias |
| [`docs/DECISION-LOG.md`](docs/DECISION-LOG.md) | Registro histórico de arquitetura e decisões (ADR) |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Como contribuir com a CLI do DARE (Rust Workspace) |

## Licença

Apache-2.0 — veja [`LICENSE`](LICENSE).
