<?php

namespace App\Llm\Providers;

use App\Llm\Contracts\LlmProvider;

class DummyProvider implements LlmProvider
{
    public function complete(string $prompt, int $maxTokens = 512): string
    {
        return '[dummy] '.substr($prompt, 0, $maxTokens);
    }
}
