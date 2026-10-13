import { describe, it, expect } from 'vitest';
import { runEcho, echoTool } from '../src/tools/echo.js';
import { TOOLS, callTool } from '../src/tools/index.js';
import { PROMPTS, getPrompt } from '../src/prompts/index.js';

describe('echo tool', () => {
  it('is registered in TOOLS', () => {
    expect(TOOLS).toContain(echoTool);
  });

  it('returns its input verbatim', async () => {
    const out = await runEcho({ text: 'hello' });
    expect(out).toBe('hello');
  });

  it('rejects empty text', async () => {
    await expect(runEcho({ text: '' })).rejects.toThrow();
  });

  it('rejects missing text', async () => {
    await expect(runEcho({})).rejects.toThrow();
  });

  it('callTool routes to echo', async () => {
    const res = await callTool('echo', { text: 'hi' });
    expect(res.content[0].text).toBe('hi');
    expect(res.isError).toBeFalsy();
  });

  it('callTool returns isError for unknown tool', async () => {
    const res = await callTool('unknown', {});
    expect(res.isError).toBe(true);
  });
});

describe('summarize prompt', () => {
  it('is registered in PROMPTS', () => {
    expect(PROMPTS.some((p) => p.name === 'summarize')).toBe(true);
  });

  it('returns a user message with the input text', async () => {
    const res = await getPrompt('summarize', { text: 'long text here' });
    expect(res.messages[0].role).toBe('user');
    expect(res.messages[0].content.text).toContain('long text here');
  });

  it('throws on unknown prompt', async () => {
    await expect(getPrompt('unknown', {})).rejects.toThrow();
  });
});
