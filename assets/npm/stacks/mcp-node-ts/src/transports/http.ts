import express from 'express';
import { randomUUID } from 'node:crypto';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';

/**
 * Streamable HTTP — modern MCP HTTP transport. Each client gets its own
 * session keyed by Mcp-Session-Id header (auto-generated on initialize).
 */
export async function startHttpTransport(server: Server, port: number): Promise<void> {
  const app = express();
  app.use(express.json());

  // Rate limit (same shape as SSE) — apply if RATE_LIMIT_RPM is set.
  const rpm = Number(process.env.RATE_LIMIT_RPM ?? 0);
  if (rpm > 0) {
    const seen = new Map<string, { count: number; resetAt: number }>();
    app.use((req, res, next) => {
      const ip = req.ip ?? 'unknown';
      const now = Date.now();
      const entry = seen.get(ip);
      if (!entry || entry.resetAt < now) {
        seen.set(ip, { count: 1, resetAt: now + 60_000 });
        return next();
      }
      if (entry.count >= rpm) {
        res.status(429).json({ error: 'rate limit exceeded' });
        return;
      }
      entry.count += 1;
      next();
    });
  }

  const transports = new Map<string, StreamableHTTPServerTransport>();

  app.post('/mcp/v1/messages', async (req, res) => {
    const sessionId = req.header('mcp-session-id');
    let transport = sessionId ? transports.get(sessionId) : undefined;
    if (!transport) {
      transport = new StreamableHTTPServerTransport({
        sessionIdGenerator: () => randomUUID(),
        onsessioninitialized: (id: string) => {
          transports.set(id, transport!);
        },
      });
      await server.connect(transport);
    }
    await transport.handleRequest(req, res, req.body);
  });

  app.get('/mcp/v1/messages', async (req, res) => {
    const sessionId = req.header('mcp-session-id');
    const transport = sessionId ? transports.get(sessionId) : undefined;
    if (!transport) {
      res.status(400).json({ error: 'no active session' });
      return;
    }
    await transport.handleRequest(req, res);
  });

  await new Promise<void>((resolve) => {
    app.listen(port, () => {
      process.stderr.write(`[mcp/http] listening on :${port}\n`);
      resolve();
    });
  });
}
