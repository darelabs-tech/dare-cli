// Smoke tests — pure tools/prompts, no SDK or transport needed.
package tests

import (
	"testing"

	"github.com/stretchr/testify/require"

	"{{moduleName}}/internal/prompts"
	"{{moduleName}}/internal/server"
	"{{moduleName}}/internal/tools"
)

func TestEchoReturnsInput(t *testing.T) {
	out, err := tools.Echo("hello")
	require.NoError(t, err)
	require.Equal(t, "hello", out)
}

func TestEchoRejectsEmpty(t *testing.T) {
	_, err := tools.Echo("")
	require.ErrorIs(t, err, tools.ErrEmpty)
}

func TestSummarizeIncludesInput(t *testing.T) {
	out := prompts.Summarize("long text here")
	require.Contains(t, out, "long text here")
	require.Contains(t, out, "Summarize")
}

func TestInventoryListsEcho(t *testing.T) {
	inv := server.InventoryJSON()
	toolsList, ok := inv["tools"].([]map[string]any)
	require.True(t, ok)
	require.NotEmpty(t, toolsList)
	require.Equal(t, "echo", toolsList[0]["name"])
}
