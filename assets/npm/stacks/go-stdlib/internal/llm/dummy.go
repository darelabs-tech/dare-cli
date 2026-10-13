package llm

import "context"

type DummyProvider struct{}

func NewDummyProvider() *DummyProvider { return &DummyProvider{} }

func (d *DummyProvider) Complete(_ context.Context, prompt string, maxTokens int) (string, error) {
	if maxTokens > 0 && len(prompt) > maxTokens {
		return "[dummy] " + prompt[:maxTokens], nil
	}
	return "[dummy] " + prompt, nil
}
