// Package server builds the mcp-go server with tools and prompts.
//
// NOTE: mark3labs/mcp-go is pre-1.0. The registration calls below match the
// v0.8.x surface; adjust here if an upstream bump changes the API. Business
// logic lives in internal/tools and internal/prompts (pure, unit-tested) so a
// breaking SDK change only touches this file.
package server

import (
	"context"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"

	"{{moduleName}}/internal/prompts"
	"{{moduleName}}/internal/tools"
)

// New builds and returns a configured MCP server.
func New() *server.MCPServer {
	s := server.NewMCPServer(
		"{{projectName}}",
		"0.1.0",
		server.WithToolCapabilities(true),
		server.WithPromptCapabilities(true),
	)

	echoTool := mcp.NewTool("echo",
		mcp.WithDescription("Returns its input. Canonical smoke test for an MCP server."),
		mcp.WithString("text",
			mcp.Required(),
			mcp.Description("Text to echo back"),
		),
	)
	s.AddTool(echoTool, func(_ context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		// v0.54: tool arguments are accessed via GetArguments() (returns map[string]any).
		text, _ := req.GetArguments()["text"].(string)
		out, err := tools.Echo(text)
		if err != nil {
			return mcp.NewToolResultError(err.Error()), nil
		}
		return mcp.NewToolResultText(out), nil
	})

	summarizePrompt := mcp.NewPrompt("summarize",
		mcp.WithPromptDescription("Summarize the given text in 1-2 sentences."),
		mcp.WithArgument("text", mcp.ArgumentDescription("Text to summarize"), mcp.RequiredArgument()),
	)
	s.AddPrompt(summarizePrompt, func(_ context.Context, req mcp.GetPromptRequest) (*mcp.GetPromptResult, error) {
		text := req.Params.Arguments["text"]
		body := prompts.Summarize(text)
		return mcp.NewGetPromptResult(
			"summarize",
			[]mcp.PromptMessage{
				mcp.NewPromptMessage(mcp.RoleUser, mcp.NewTextContent(body)),
			},
		), nil
	})

	return s
}

// InventoryJSON returns a machine-readable tool inventory for --list-tools (M-03).
func InventoryJSON() map[string]any {
	return map[string]any{
		"tools": []map[string]any{
			{
				"name":        "echo",
				"description": "Returns its input. Canonical smoke test for an MCP server.",
				"inputSchema": map[string]any{
					"type":     "object",
					"required": []string{"text"},
					"properties": map[string]any{
						"text": map[string]any{"type": "string", "minLength": 1},
					},
				},
			},
		},
	}
}
