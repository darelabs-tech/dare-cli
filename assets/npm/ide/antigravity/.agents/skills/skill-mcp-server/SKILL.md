---
name: skill-mcp-server
description: Padrões DARE para servidores MCP (Model Context Protocol) em TypeScript ou Python. Define tools, resources, prompts; suporta transports stdio, SSE e HTTP; validação Zod/Pydantic; autorização por tool; tracing estruturado; testes; publicação no MCP registry.
---

# DARE MCP Server Skill

Você é um especialista em servidores MCP (Model Context Protocol da Anthropic). Seu papel é gerar servidores MCP **bem estruturados, seguros, testáveis e publicáveis**, expondo tools/resources/prompts via stdio, SSE ou HTTP.

## Quando usar

- Você precisa expor capacidades para Claude Code, Cursor ou outro cliente MCP
- Você quer integrar uma API externa (Linear, GitHub, banco interno) como tool MCP
- Você está auditando um servidor MCP existente

## O que é MCP em 30 segundos

MCP (Model Context Protocol) é um protocolo aberto da Anthropic para conectar agentes a fontes de dados e ações externas. Um servidor MCP expõe:

- **Tools** — funções chamáveis pelo agente (ex: `create_issue`, `query_db`)
- **Resources** — dados leitáveis (ex: documentos, schemas)
- **Prompts** — templates parametrizados

Transports suportados:
- **stdio** — agente spawnea processo local, comunica via stdin/stdout (mais simples)
- **SSE** — server remoto, agente conecta via Server-Sent Events
- **HTTP** — server remoto stateless

## Stack canônica (TypeScript)

- **Node 20+** com TypeScript 5.5+
- **@modelcontextprotocol/sdk** SDK oficial
- **Zod** para validação de input/output de tools
- **vitest** para testes
- **eslint + prettier**

## Stack canônica (Python)

- **Python 3.11+**
- **mcp** SDK oficial (`pip install mcp`)
- **Pydantic v2** para schemas
- **pytest + pytest-asyncio**
- **ruff + mypy**

## Estrutura recomendada (TypeScript)

```
mcp-meu-server/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts                ← entrypoint (stdio)
│   ├── server.ts               ← criação do MCP Server
│   ├── tools/
│   │   ├── create_issue.ts
│   │   └── search_issues.ts
│   ├── resources/
│   │   └── schema.ts
│   ├── prompts/
│   │   └── issue_template.ts
│   ├── schemas/                ← Zod schemas
│   └── lib/
│       └── linear-client.ts    ← cliente da API externa
└── tests/
```

## Exemplo: server stdio TypeScript

```typescript
// src/index.ts
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { registerTools } from './tools/index.js';
import { registerResources } from './resources/index.js';

const server = new Server(
  { name: 'meu-mcp-server', version: '1.0.0' },
  { capabilities: { tools: {}, resources: {}, prompts: {} } }
);

registerTools(server);
registerResources(server);

const transport = new StdioServerTransport();
await server.connect(transport);
```

## Definindo uma tool

```typescript
// src/tools/create_issue.ts
import { z } from 'zod';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

const CreateIssueInput = z.object({
  title: z.string().min(1).max(255),
  description: z.string().optional(),
  priority: z.enum(['low', 'medium', 'high']).default('medium'),
  team_id: z.string().uuid(),
});

const CreateIssueOutput = z.object({
  id: z.string(),
  url: z.string().url(),
  number: z.number(),
});

export function register(server: Server) {
  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: [{
      name: 'create_issue',
      description: 'Create a new issue in Linear',
      inputSchema: CreateIssueInput.shape,  // ou gerar via zod-to-json-schema
    }],
  }));

  server.setRequestHandler(CallToolRequestSchema, async (req) => {
    if (req.params.name !== 'create_issue') {
      throw new Error('unknown tool');
    }
    const input = CreateIssueInput.parse(req.params.arguments);
    const result = await linearClient.createIssue(input);
    const output = CreateIssueOutput.parse({
      id: result.id,
      url: result.url,
      number: result.number,
    });
    return { content: [{ type: 'text', text: JSON.stringify(output) }] };
  });
}
```

## Exemplo: server stdio Python

```python
# server.py
import asyncio
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field

app = Server("meu-mcp-server")

class CreateIssueInput(BaseModel):
    title: str = Field(min_length=1, max_length=255)
    description: str | None = None
    priority: str = Field(default="medium", pattern="^(low|medium|high)$")
    team_id: str

@app.list_tools()
async def list_tools() -> list[Tool]:
    return [
        Tool(
            name="create_issue",
            description="Create a new issue in Linear",
            inputSchema=CreateIssueInput.model_json_schema(),
        )
    ]

@app.call_tool()
async def call_tool(name: str, arguments: dict) -> list[TextContent]:
    if name != "create_issue":
        raise ValueError(f"unknown tool: {name}")
    input_data = CreateIssueInput.model_validate(arguments)
    result = await linear_client.create_issue(input_data)
    return [TextContent(type="text", text=result.model_dump_json())]

async def main():
    async with stdio_server() as (read, write):
        await app.run(read, write, app.create_initialization_options())

if __name__ == "__main__":
    asyncio.run(main())
```

## Resources

```typescript
import { ListResourcesRequestSchema, ReadResourceRequestSchema } from '@modelcontextprotocol/sdk/types.js';

server.setRequestHandler(ListResourcesRequestSchema, async () => ({
  resources: [{
    uri: 'linear://schema',
    name: 'Linear Issue Schema',
    mimeType: 'application/json',
  }],
}));

server.setRequestHandler(ReadResourceRequestSchema, async (req) => {
  if (req.params.uri === 'linear://schema') {
    return { contents: [{ uri: req.params.uri, mimeType: 'application/json', text: JSON.stringify(schema) }] };
  }
  throw new Error('unknown resource');
});
```

## Prompts

```typescript
import { ListPromptsRequestSchema, GetPromptRequestSchema } from '@modelcontextprotocol/sdk/types.js';

server.setRequestHandler(ListPromptsRequestSchema, async () => ({
  prompts: [{
    name: 'triage_issue',
    description: 'Triage an issue and assign priority',
    arguments: [
      { name: 'description', description: 'Issue description', required: true },
    ],
  }],
}));

server.setRequestHandler(GetPromptRequestSchema, async (req) => ({
  messages: [{
    role: 'user',
    content: { type: 'text', text: `Triage this issue and suggest a priority:\n\n${req.params.arguments?.description}` },
  }],
}));
```

## Transports

| Transport | Quando usar | Setup |
|---|---|---|
| **stdio** | Server local, agente spawnea processo | `StdioServerTransport` |
| **SSE** | Server remoto, streaming bidirectional | `SSEServerTransport` + Express/Fastify |
| **HTTP** | Server remoto stateless | endpoint POST que devolve JSON |

## Autorização por tool

Toda tool deve declarar:
- **Quem pode chamar** — escopo ou role
- **Quais argumentos validar** — não confiar no client
- **Side effects** — destrutivo? idempotente?

```typescript
// tool destrutiva exige confirmação ou role admin
if (input.confirm !== 'YES_I_AM_SURE') {
  throw new Error('this tool requires confirmation');
}
if (!callerHasRole('admin')) {
  throw new Error('admin only');
}
```

## Observabilidade

Logue cada tool call (sem dados sensíveis):

```typescript
console.error(JSON.stringify({
  ts: new Date().toISOString(),
  tool: req.params.name,
  duration_ms: Date.now() - start,
  success: true,
  // sem args (PII), sem result (PII)
}));
```

> stdio usa stdout para protocolo MCP — **logs sempre em stderr**.

## Testes

```typescript
// tests/create_issue.test.ts
import { test, expect, vi } from 'vitest';
import { CreateIssueInput } from '../src/tools/create_issue.js';

test('valida input correto', () => {
  const valid = CreateIssueInput.parse({
    title: 'Test',
    team_id: '550e8400-e29b-41d4-a716-446655440000',
  });
  expect(valid.priority).toBe('medium');  // default
});

test('rejeita team_id inválido', () => {
  expect(() => CreateIssueInput.parse({
    title: 'Test',
    team_id: 'not-a-uuid',
  })).toThrow();
});
```

E2E com cliente MCP de teste:

```typescript
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const transport = new StdioClientTransport({ command: 'node', args: ['dist/index.js'] });
const client = new Client({ name: 'test', version: '0.0.0' }, { capabilities: {} });
await client.connect(transport);

const tools = await client.listTools();
expect(tools.tools.map(t => t.name)).toContain('create_issue');
```

## Antipatterns

| AP | Antipattern | Por quê |
|---|---|---|
| AP-01 | Logs no stdout (stdio) | Quebra o protocolo — sempre stderr |
| AP-02 | Tool sem schema de input | Cliente vai passar lixo |
| AP-03 | Sem validação no server | Confia no client (errado) |
| AP-04 | Tool destrutiva sem confirmação | Agente pode deletar dados sem querer |
| AP-05 | Secrets em código | Use env vars |
| AP-06 | Tool name não-descritivo (`do_thing`) | Agente não sabe quando usar |
| AP-07 | Description vago | Agente erra na escolha |
| AP-08 | Resource sem mimeType | Cliente não sabe parsear |

## Publicação

### Pré-requisitos para registry

- README.md com:
  - Como instalar/configurar
  - Lista de tools/resources/prompts
  - Variáveis de ambiente necessárias
  - Exemplo de `claude_desktop_config.json` (stdio)
- LICENSE (MIT recomendado)
- `package.json` ou `pyproject.toml` versionado
- Schema JSON exposto (vem das definições Zod/Pydantic)

### Distribuição

- **npm** — `npm publish` para servers TS
- **PyPI** — `python -m build && twine upload` para Python
- **Docker** — para SSE/HTTP servers (multi-stage build)

## Configuração no cliente (exemplo Claude Desktop)

```json
{
  "mcpServers": {
    "meu-server": {
      "command": "node",
      "args": ["/abs/path/to/dist/index.js"],
      "env": {
        "LINEAR_API_KEY": "lin_api_..."
      }
    }
  }
}
```

## Como aplicar

### Passo 1: Decidir transport

Local + uso pessoal → **stdio**. Multi-usuário ou cloud → **SSE/HTTP**.

### Passo 2: Listar tools/resources/prompts

Inventarie no DESIGN.md o que vai expor. Cada tool: nome, descrição clara, input schema, output schema.

### Passo 3: Scaffold

Use SDK oficial. Estruture em `tools/`, `resources/`, `prompts/`.

### Passo 4: Schemas Zod/Pydantic

Toda tool tem schema de input. Output validado antes de retornar.

### Passo 5: Logs em stderr + observabilidade

`console.error` (stdio) com JSON estruturado.

### Passo 6: Testes + publicação

Vitest/pytest. README + LICENSE MIT. Publicar npm/PyPI.

## Dicas

- **Combine** com `dare-llm-integration` se o server chama LLM
- **Use** `dare-security` para auditar dependências
- **Para tools destrutivas**, exija confirm flag explícito

---

Esta skill é parte do DARE Method e está sob licença MIT.
