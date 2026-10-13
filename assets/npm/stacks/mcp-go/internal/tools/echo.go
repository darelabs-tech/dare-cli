// Package tools holds pure tool implementations, testable without the SDK.
package tools

import "errors"

// ErrEmpty is returned when echo is given an empty string.
var ErrEmpty = errors.New("text must be a non-empty string")

// Echo returns its input verbatim.
func Echo(text string) (string, error) {
	if text == "" {
		return "", ErrEmpty
	}
	return text, nil
}
