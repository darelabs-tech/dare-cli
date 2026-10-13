import type { Prompt } from '@modelcontextprotocol/sdk/types.js';

export const PROMPTS: Prompt[] = [
  {
    name: 'summarize',
    description: 'Summarize the given text in 1-2 sentences.',
    arguments: [
      {
        name: 'text',
        description: 'Text to summarize',
        required: true,
      },
    ],
  },
];

export async function getPrompt(
  name: string,
  args: Record<string, unknown>,
): Promise<{ messages: Array<{ role: 'user'; content: { type: 'text'; text: string } }> }> {
  if (name === 'summarize') {
    const text = String(args.text ?? '');
    return {
      messages: [
        {
          role: 'user',
          content: {
            type: 'text',
            text: `Summarize the following text in 1-2 sentences.\n\n${text}`,
          },
        },
      ],
    };
  }
  throw new Error(`Unknown prompt: ${name}`);
}
