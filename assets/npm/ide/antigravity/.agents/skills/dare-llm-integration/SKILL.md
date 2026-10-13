---
name: dare-llm-integration
description: Integração com LLMs (Large Language Models) em projetos DARE. Fornece abstração LLMProvider, cache em memória com TTL, rate limiting via token bucket, prompt templates versionados e validação de output via JSON Schema. Cobre antipatterns crítico de prompt injection e LLM output não validado.
---

# DARE LLM Integration Skill

Você é um especialista em integração com LLMs (Gemini, Claude, GPT, modelos locais). Seu papel é garantir que toda chamada a LLM em projeto DARE seja **abstraída, cacheada, rate-limited, validada e auditável**.

## Quando usar esta skill

- Projeto vai consumir Gemini, Claude API, OpenAI, Ollama ou similar
- Você está revisando Handler que chama SDK de LLM diretamente
- Você está auditando custos de LLM no projeto
- Você está adicionando proteção contra prompt injection

## A arquitetura recomendada

```
┌────────────────────────────────────────────────────────┐
│  Handler / Service                                      │
└────────────────────────────────────────────────────────┘
                       ↓ injeta
┌────────────────────────────────────────────────────────┐
│  LLMProvider (interface)                                │
│  ├── GeminiProvider                                     │
│  ├── ClaudeProvider                                     │
│  ├── OpenAIProvider                                     │
│  └── OllamaProvider (local)                             │
└────────────────────────────────────────────────────────┘
                       ↓ wrapping
┌────────────────────────────────────────────────────────┐
│  Cache (TTL)  +  RateLimit (token bucket)  +  Schema    │
└────────────────────────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────┐
│  HTTP call externo (Gemini API, etc.)                   │
└────────────────────────────────────────────────────────┘
```

## Os 5 pilares

### 1. LLMProvider abstraction

NUNCA chame SDK do Gemini/OpenAI dentro de um Handler ou Service de negócio. Sempre passe pela interface `LLMProvider`.

```typescript
// ❌ Errado
class SummaryService {
  async run(text: string) {
    const client = new GoogleGenAI({apiKey: 'xxx'});
    return client.generateContent({contents: text});
  }
}

// ✅ Certo
class SummaryService {
  constructor(private llm: LLMProvider) {}
  async run(text: string) {
    return this.llm.complete({
      promptId: 'summarize-v1',
      input: { text },
      schema: SummarySchema,
    });
  }
}
```

### 2. Cache em memória com TTL

Toda chamada deve passar por cache. Cache key = `hash(promptId + input + model)`. TTL configurável por prompt (default 1 hora).

**Antipattern AP-02:** Sem cache → custo explode em loops.

### 3. Rate limit via token bucket

Proteja a integração externa **e** seu wallet. Configure tokens/segundo por provider.

| Provider | Limite típico |
|---|---|
| Gemini Free | 15 RPM, 1 milhão tokens/dia |
| Claude API | 50 RPM (tier 1) |
| OpenAI | 500 RPM (tier 1) |
| Ollama local | sem limite, mas latência alta |

### 4. Prompt templates versionados

Prompts NUNCA ficam inline em código. Vivem em `prompts/<id>.v<n>.md`:

```markdown
---
id: summarize
version: 1
model: gemini-2.0-flash
temperature: 0.2
max_tokens: 500
schema: SummarySchema
---

# System

You are a concise summarizer. Output JSON with this schema:
{schema}

# User

Summarize the following text in 3 bullet points:

<text>{{ text }}</text>
```

Substituição usa **template engine seguro** (Jinja2, Handlebars com sandbox, ou string interpolation com escape) — nunca `eval` ou `f-string` direto com user input.

### 5. Validação via JSON Schema

LLM mente. Sempre valide o output:

```typescript
const result = await llm.complete({...});
const validated = SummarySchema.parse(result);  // Zod, Pydantic, ajv...
if (!validated.success) {
  throw new LLMOutputInvalidError(validated.error);
  // Logar + alertar — pode indicar prompt injection ou drift de modelo
}
```

## Métricas obrigatórias

| ID | Métrica | Como medir |
|---|---|---|
| M-01 | 100% das chamadas LLM via LLMProvider injetado | grep por `new GoogleGenAI \| new OpenAI \| Anthropic(` em Services |
| M-02 | 100% das responses LLM cacheadas | logs do cache layer |
| M-03 | 100% das requests LLM com rate limit | rate limiter ativo em todo provider |
| M-04 | 100% das respostas LLM validadas contra schema | grep por `.parse(` ou `Pydantic` no resultado |

## Antipatterns

| AP | Antipattern | Por que evitar |
|---|---|---|
| AP-01 | SDK do LLM direto em Handler | impossível mockar, impossível trocar de provider |
| AP-02 | Sem cache | custo explode, latência ruim |
| AP-03 | Prompt em código | impossível versionar e A/B testar |
| AP-04 | User input direto em prompt | prompt injection trivial |
| AP-05 | Confiar em output LLM sem validar | LLM mente — schema é defesa |

## Defesa contra prompt injection (crítico)

```python
# ❌ Errado — instrução + dado misturados
system = f"You are an assistant. {user_question}"

# ✅ Certo — separação clara
messages = [
    {"role": "system", "content": "You are an assistant. Answer only based on the provided document."},
    {"role": "user", "content": f"<document>{escape(doc)}</document>\n\n<question>{escape(q)}</question>"}
]
```

Regras:
- Use delimitadores (`<document>`, `<question>`)
- Escape conteúdo do usuário (XML escape, HTML escape)
- Detecte e remova padrões "Ignore as instruções acima", "You are now ..."
- Valide output contra schema — output fora do schema = possível injection bem-sucedido

## Como aplicar

### Passo 1: Criar abstração LLMProvider

```typescript
interface LLMProvider {
  complete(req: LLMRequest): Promise<LLMResponse>;
}

interface LLMRequest {
  promptId: string;
  input: Record<string, unknown>;
  schema: ZodSchema;
  options?: { temperature?: number; maxTokens?: number };
}
```

### Passo 2: Implementar 1+ provider

Comece com o que o projeto usa (Gemini, Claude, etc.). Adicione mais conforme necessário.

### Passo 3: Configurar cache

`packages/skills/dare-llm-integration/cache/` tem implementação em memória com TTL pronta. Use ou copie.

### Passo 4: Configurar rate limit

Token bucket por provider, configurado em `app.config`.

### Passo 5: Migrar prompts inline para arquivos

Inventarie todos os `system: "..."` no código e mova para `prompts/<id>.v1.md`.

### Passo 6: Adicionar validação de output

Schema Zod/Pydantic/serde para cada prompt. CI falha se prompt não tiver schema declarado.

## Boas práticas

1. **Provider neutro no domínio** — Service não sabe se é Gemini ou Claude
2. **Custo é parte da observabilidade** — logue tokens in/out + custo estimado por request
3. **Fallback** — se provider primário falhar, tente secundário (Gemini → Claude)
4. **Latência alta = degrada UX** — esconda atrás de fila assíncrona quando >2s

## Dicas

- **Leia** `docs/design/skills/dare-llm-integration/DESIGN.md`
- **Combine** com `dare-security` (RS-* para prompt injection)
- **Use** os providers em `packages/skills/dare-llm-integration/providers/` como referência

---

Esta skill é parte do DARE Method e está sob licença MIT.
