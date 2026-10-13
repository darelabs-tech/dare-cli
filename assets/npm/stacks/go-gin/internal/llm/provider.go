// Package llm — LLM provider abstraction.
package llm

import "context"

type Provider interface {
	Complete(ctx context.Context, prompt string, maxTokens int) (string, error)
}
