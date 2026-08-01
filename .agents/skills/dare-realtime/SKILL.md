---
name: dare-realtime
description: Comunicação real-time (WebSocket, SSE) em projetos DARE. Fornece schema validation de eventos, registro central de tipos, reconexão com exponential backoff, e gerenciamento de subscriptions com limpeza garantida (zero ghost listeners).
---

# DARE Realtime Skill

Você é um especialista em comunicação real-time. Seu papel é garantir que toda integração WebSocket/SSE em projeto DARE seja **tipada, autorizada, reconectável e sem ghost listeners**.

## Quando usar esta skill

- Projeto precisa de WebSocket (chat, notifications, live updates)
- Projeto precisa de SSE (logs streaming, métricas em tempo real)
- Você está auditando ghost listeners (memória crescendo após desconexão)
- Você está revisando código que faz `socket.on('event', ...)` sem cleanup

## Arquitetura recomendada

```
┌─────────────────────────────────────────────────────────┐
│  Event Registry                                          │
│  - Tipos de evento com schema JSON                       │
│  - Versionamento (v1, v2…)                               │
│  - Autorização por evento                                │
└─────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│  Subscription Manager                                    │
│  - Map<connection_id, Set<event_type>>                   │
│  - Cleanup garantido on disconnect                       │
└─────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│  Reconnect Strategy                                      │
│  - Exponential backoff (1s, 2s, 4s, 8s, max 30s)         │
│  - Jitter para evitar thundering herd                    │
│  - Resync de estado após reconectar                      │
└─────────────────────────────────────────────────────────┘
```

## Os 4 pilares

### 1. Schema validation de eventos

Todo evento WebSocket/SSE tem schema JSON:

```typescript
// event_registry.ts
const events = {
  'user.created.v1': z.object({
    id: z.string().uuid(),
    email: z.string().email(),
    createdAt: z.string().datetime(),
  }),
  'message.sent.v1': z.object({
    conversationId: z.string().uuid(),
    senderId: z.string().uuid(),
    body: z.string().max(2000),
    sentAt: z.string().datetime(),
  }),
};
```

Server valida antes de emitir, client valida antes de processar. Evento inválido = log + drop.

### 2. Registro central de tipos

Um único `event_registry.ts/py/rs` lista todos os event types. Adicionar evento novo passa por PR explícito. Versionamento via sufixo (`.v1`, `.v2`).

### 3. Reconexão com exponential backoff

```typescript
class ReconnectStrategy {
  private attempt = 0;
  next(): number {
    const base = Math.min(1000 * 2 ** this.attempt, 30_000);
    const jitter = Math.random() * 1000;
    this.attempt++;
    return base + jitter;
  }
  reset() { this.attempt = 0; }
}
```

Após reconectar, **resync de estado** — buscar eventos perdidos via REST ou pedir snapshot via WS.

### 4. Subscription manager (zero ghost listeners)

Cada subscription rastreada em map, com cleanup automático:

```typescript
class SubscriptionManager {
  private subs = new Map<string, Set<string>>(); // connectionId → events
  
  subscribe(connId: string, event: string) {
    if (!this.subs.has(connId)) this.subs.set(connId, new Set());
    this.subs.get(connId)!.add(event);
  }
  
  unsubscribeAll(connId: string) {
    this.subs.delete(connId);
  }
  
  // Forçar limpeza on disconnect — SEMPRE
  onDisconnect(connId: string) {
    this.unsubscribeAll(connId);
  }
}
```

## Métricas obrigatórias

| ID | Métrica | Como medir |
|---|---|---|
| M-01 | 100% de event types com JSON Schema definido | grep no registry |
| M-02 | 100% de subscriptions autorizadas | verificar `authorize()` antes de `subscribe()` |
| M-03 | 0 ghost listeners após desconexão | métrica `subs.size` por conexão = 0 após disconnect |
| M-04 | Estratégia de reconexão configurada | grep por `ReconnectStrategy` ou similar |

## Antipatterns

| AP | Antipattern | Sinal | Correção |
|---|---|---|---|
| AP-01 | Evento sem schema | `socket.emit('foo', anything)` | Adicionar schema no registry |
| AP-02 | `socket.on(...)` sem cleanup | listener fica vivo após disconnect | Usar SubscriptionManager |
| AP-03 | Reconexão sem backoff | reconecta imediatamente em loop | Exponential backoff + jitter |
| AP-04 | Sem autorização de subscription | usuário ouve eventos de outro tenant | `authorize(user, event)` antes de `subscribe()` |
| AP-05 | Broadcast sem filtro | `io.emit()` para todos | Use rooms / channels por tenant |
| AP-06 | Estado não recuperado após reconectar | UI mostra dados stale | Resync via REST ou snapshot |

## Como aplicar

### Passo 1: Criar event registry

```typescript
// src/realtime/event_registry.ts
export const EventRegistry = {
  'user.created.v1': { schema: UserCreatedSchema, requires: 'admin' },
  'message.sent.v1': { schema: MessageSentSchema, requires: 'participant' },
};
```

### Passo 2: Implementar subscription manager

Use `packages/skills/dare-realtime/subscription_manager.ts` como template. Garante cleanup on disconnect.

### Passo 3: Configurar reconexão no client

```typescript
const strategy = new ReconnectStrategy({ base: 1000, max: 30_000 });
socket.on('disconnect', () => {
  setTimeout(() => socket.connect(), strategy.next());
});
socket.on('connect', () => {
  strategy.reset();
  resyncState(); // fetch via REST eventos perdidos
});
```

### Passo 4: Adicionar autorização

```typescript
socket.on('subscribe', (eventType) => {
  if (!authorize(socket.user, eventType)) {
    socket.emit('error', { code: 'FORBIDDEN' });
    return;
  }
  subscriptionManager.subscribe(socket.id, eventType);
});
```

### Passo 5: Validar eventos no client e server

Server antes de emitir:
```typescript
const schema = EventRegistry[eventType].schema;
const valid = schema.safeParse(payload);
if (!valid.success) { log.error(...); return; }
io.to(room).emit(eventType, valid.data);
```

Client antes de processar:
```typescript
socket.on(eventType, (payload) => {
  const valid = EventRegistry[eventType].schema.safeParse(payload);
  if (!valid.success) { log.warn(...); return; }
  handleEvent(valid.data);
});
```

## Boas práticas

1. **WebSocket vs SSE** — SSE para server→client unidirecional (logs, métricas). WS para bidirectional (chat).
2. **Rooms/channels por tenant** — evita broadcast cross-tenant
3. **Heartbeat / ping-pong** — detectar conexão zumbi (network OK mas peer não responde)
4. **Backpressure** — se client lento, fila no server explode. Drop eventos antigos ou aplicar rate limit.
5. **Replay limitado** — guardar últimos N eventos por room para reconexão recente, não histórico completo

## Stack recomendada

| Stack | WebSocket | SSE |
|---|---|---|
| Node | `socket.io`, `ws` | `eventsource` (client) + endpoint custom |
| Rails | Action Cable | Rack streaming |
| Rust/Axum | `tokio-tungstenite`, `axum` ws | `axum::sse::Sse` |
| FastAPI | `fastapi.WebSocket` | `EventSourceResponse` |
| Go | `gorilla/websocket`, `melody` | stdlib `http.Flusher` |

## Dicas

- **Leia** `docs/design/skills/dare-realtime/DESIGN.md`
- **Combine** com `dare-security` para autorização
- **Use** os templates em `packages/skills/dare-realtime/`

---

Esta skill é parte do DARE Method e está sob licença MIT.
