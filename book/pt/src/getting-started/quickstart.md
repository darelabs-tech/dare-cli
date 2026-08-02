# Quickstart

Este guia cobre tudo o que você precisa para criar seu primeiro projeto com o DARE CLI em menos de 5 minutos.

---

## Pré-requisito

Tenha o DARE CLI instalado. Se ainda não instalou:

```bash
curl -fsSL https://darelabs.tech/install | sh
```

---

## Passo 1 — Criar um projeto novo

```bash
# Criar pasta e entrar
mkdir meu-projeto && cd meu-projeto

# Inicializar o DARE
dare init
```

O comando `dare init` em modo interativo vai te perguntar:

1. **Nome do projeto** (ex.: `meu-api`)
2. **Stack** — ex.: `rust`, `python`, `node`, `laravel`, `go`, `rails`
3. **MCP transport** (opcional) — se quiser um MCP server

Para inicializar sem interatividade (CI/scripts):

```bash
dare init --stack rust --non-interactive
```

---

## Passo 2 — Bootstrapar a stack

Após o `dare init`, aplique o scaffold da stack escolhida:

```bash
dare bootstrap
```

Isso materializa a estrutura de diretórios, arquivos de configuração e harnesses para os agentes de IA instalados.

---

## Passo 3 — Criar o Design

Descreva o que você quer construir:

```bash
dare design "Quero uma API REST de autenticação JWT com refresh token em Rust"
```

Isso gera `DARE/DESIGN.md` — o documento de requisitos que guia todo o desenvolvimento.

---

## Passo 4 — Gerar o Blueprint

Com o Design aprovado, a IA propõe a arquitetura:

```bash
dare blueprint
```

Isso gera `DARE/BLUEPRINT.md` com:
- Arquitetura de camadas
- Endpoints e contratos
- Modelo de dados
- Decomposição em tasks

**Revise o Blueprint antes de prosseguir.**

---

## Passo 5 — Executar as Tasks

Com o Blueprint aprovado, inicie a implementação task por task:

```bash
dare execute task-001
```

O **Ralph Loop** entra em ação: a IA implementa, roda os gates de validação (testes, linter, formatter) e itera automaticamente até todos passarem.

Para ver o status do grafo de execução:

```bash
dare dag status
```

---

## O que foi criado?

Após o `dare init`, seu projeto terá a seguinte estrutura:

```
meu-projeto/
├── dare.config.json         ← configuração do projeto
├── DARE/
│   ├── DESIGN.md            ← requisitos (fase 1)
│   ├── BLUEPRINT.md         ← arquitetura (fase 2)
│   ├── TASKS.md             ← lista de tasks
│   └── dare-dag.yaml        ← grafo de execução
└── .agents/
    ├── AGENTS.md            ← contexto para agentes de IA
    └── skills/              ← skills do DARE para cada agente
```

---

## Próximos passos

- [O Método DARE completo](../method/overview.md) — entenda as 4 fases em profundidade
- [Referência de Comandos](../commands/overview.md) — todos os comandos disponíveis
- [Integração com Agentes](../agents/antigravity.md) — configurar Antigravity, Cursor ou Claude Code
