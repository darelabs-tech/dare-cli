<?php

namespace App\Llm\Providers;

use App\Llm\Contracts\LlmProvider;
use Illuminate\Support\Facades\Http;
use RuntimeException;

class OpenAiProvider implements LlmProvider
{
    public function __construct(
        private readonly string $apiKey,
        private readonly string $model = 'gpt-4o-mini',
        private readonly string $baseUri = 'https://api.openai.com/v1',
    ) {
    }

    public function complete(string $prompt, int $maxTokens = 512): string
    {
        $res = Http::withToken($this->apiKey)
            ->post("{$this->baseUri}/chat/completions", [
                'model' => $this->model,
                'messages' => [['role' => 'user', 'content' => $prompt]],
                'max_tokens' => $maxTokens,
            ]);

        if (! $res->successful()) {
            throw new RuntimeException('OpenAI request failed: '.$res->status());
        }

        return (string) ($res->json('choices.0.message.content') ?? '');
    }
}
