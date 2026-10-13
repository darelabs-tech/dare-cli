import express from 'express';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { SSEServerTransport } from '@modelcontextprotocol/sdk/server/sse.js';

/**
 * Legacy MCP SSE transport. Holds one SSE connection at a time; multiple
 * clients require running multiple server instances behind a load balancer.
 *
 * For new deployments prefer the modern Streamable HTTP transport
 * (transports/http.ts) — SSE is kept here for compatibility with older
 * MCP clients that only speak the SSE wire format.
 */
export async function startSseTransport(server: Server, port: number): Promise<void> {
  const app = express();

  // Rate limit knob — apply if RATE_LIMIT_RPM is set. Simple token bucket per IP.
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

  let transport: SSEServerTransport | null = null;

  app.get('/mcp/v1/sse', async (_req, res) => {
    transport = new SSEServerTransport('/mcp/v1/messages', res);
    await server.connect(transport);
  });

  app.post('/mcp/v1/messages', express.json(), async (req, res) => {
    if (!transport) {
      res.status(400).json({ error: 'no active SSE session' });
      return;
    }
    await transport.handlePostMessage(req, res, req.body);
  });

  await new Promise<void>((resolve) => {
    app.listen(port, () => {
      process.stderr.write(`[mcp/sse] listening on :${port}\n`);
      resolve();
    });
  });
}
