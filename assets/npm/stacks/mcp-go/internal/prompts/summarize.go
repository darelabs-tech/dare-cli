// Package prompts holds pure prompt builders.
package prompts

import "fmt"

// Summarize returns the user-message body for a summarize prompt.
func Summarize(text string) string {
	return fmt.Sprintf("Summarize the following text in 1-2 sentences.\n\n%s", text)
}
