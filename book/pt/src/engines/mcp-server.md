# MCP Server

O **MCP Server embutido** do DARE CLI expõe contexto arquitetural via **Model Context Protocol** — permitindo que agentes de IA consultem o grafo de conhecimento do projeto sem precisar receber o `BLUEPRINT.md` completo.

## O que é o MCP?

O **Model Context Protocol** (MCP) é um protocolo aberto para que ferramentas (como o DARE CLI) exponham contexto estruturado para agentes de IA de forma eficiente.

## Por que MCP no DARE?

Sem MCP, um agente precisaria receber o `BLUEPRINT.md` inteiro (pode ter 30KB+) em cada mensagem. Com o MCP Server, o agente **consulta apenas o que precisa**:

| Abordagem | Tokens por consulta |
|---|---|
| Expor BLUEPRINT.md completo | ~8.000–25.000 tokens |
| MCP (consulta pontual) | ~200–800 tokens |

Economia de até **95% dos tokens**.

## Como ativar

O MCP Server do DARE é um **transporte separado** — não substitui o REST API nem o CLI.

```bash
dare mcp start              # inicia o server MCP
dare mcp start --stdio      # transporte stdio (padrão para IDEs)
dare mcp start --sse        # transporte SSE
dare mcp start --http       # transporte HTTP (porta 3777)
```

## Tools expostas via MCP

| Tool | Descrição |
|---|---|
| `dare/graph_query` | Busca no GraphRAG por query de texto |
| `dare/task_spec` | Retorna a spec completa de uma task |
| `dare/dag_status` | Status atual de todas as tasks |
| `dare/project_info` | Metadados do projeto (stack, versão, etc.) |
| `dare/blueprint_section` | Seção específica do BLUEPRINT.md |
| `dare/design_requirements` | Lista de requisitos do DESIGN.md |

## Configuração nas IDEs

### Antigravity / Gemini CLI

Adicione ao `settings.json`:

```json
{
  "mcpServers": {
    "dare": {
      "command": "dare",
      "args": ["mcp", "start", "--stdio"]
    }
  }
}
```

### Claude Code

```json
// .claude/mcp.json
{
  "mcpServers": {
    "dare": {
      "command": "dare",
      "args": ["mcp", "start", "--stdio"]
    }
  }
}
```

### Cursor

```json
// .cursor/mcp.json
{
  "mcpServers": {
    "dare": {
      "command": "dare",
      "args": ["mcp", "start", "--stdio"]
    }
  }
}
```

## Exemplo de uso pelo agente

O agente pode chamar:

```
dare/graph_query("JWT authentication")
```

E receber apenas:

```json
{
  "results": [
    { "id": "RF-01", "type": "requirement", "summary": "JWT auth with refresh" },
    { "id": "task-003", "type": "task", "status": "done", "file": "src/auth/jwt.rs" }
  ],
  "tokens_used": 312
}
```

Em vez de carregar o BLUEPRINT.md inteiro com ~15.000 tokens.

## Transporte HTTP (REST compatível)

O modo HTTP expõe uma API REST local compatível com o protocolo MCP:

```bash
dare mcp start --http --port 3777
```

```
GET  http://localhost:3777/tools          # lista tools disponíveis
POST http://localhost:3777/call           # chama uma tool
```

> **Importante (ADR-004):** REST e MCP são transportes **distintos**. O MCP server não substitui o REST API e vice-versa.
