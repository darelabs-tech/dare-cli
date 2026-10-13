import { z } from 'zod';
import type { Tool } from '@modelcontextprotocol/sdk/types.js';

export const echoInput = z.object({
  text: z.string().min(1, 'text is required'),
});

export const echoTool: Tool = {
  name: 'echo',
  description: 'Returns its input. Canonical smoke test for an MCP server.',
  inputSchema: {
    type: 'object',
    required: ['text'],
    properties: {
      text: { type: 'string', minLength: 1, description: 'Text to echo back' },
    },
  },
};

export async function runEcho(args: unknown): Promise<string> {
  const parsed = echoInput.parse(args);
  return parsed.text;
}
