import type { Tool } from '@modelcontextprotocol/sdk/types.js';
import { echoTool, runEcho } from './echo.js';

export const TOOLS: Tool[] = [echoTool];

export async function callTool(
  name: string,
  args: Record<string, unknown>,
): Promise<{ content: Array<{ type: 'text'; text: string }>; isError?: boolean }> {
  if (name === echoTool.name) {
    const text = await runEcho(args);
    return { content: [{ type: 'text', text }] };
  }
  return {
    content: [{ type: 'text', text: `Unknown tool: ${name}` }],
    isError: true,
  };
}
