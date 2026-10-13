<?php

namespace App\Llm\Contracts;

interface LlmProvider
{
    /**
     * Complete a prompt. Implementations decide on streaming vs. one-shot
     * via their own configuration; this contract is the synchronous shape.
     */
    public function complete(string $prompt, int $maxTokens = 512): string;
}
