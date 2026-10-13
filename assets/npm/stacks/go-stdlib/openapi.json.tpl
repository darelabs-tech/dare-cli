{
  "openapi": "3.0.3",
  "info": {
    "title": "{{projectName}}",
    "version": "0.1.0",
    "description": "DARE-shaped Go API on net/http stdlib."
  },
  "paths": {
    "/auth/login": {
      "post": {
        "summary": "Exchange email+password for JWT",
        "responses": { "200": { "description": "JWT issued" }, "401": { "description": "Invalid credentials" } }
      }
    },
    "/auth/me": {
      "get": {
        "summary": "Current user",
        "security": [{ "bearer": [] }],
        "responses": { "200": { "description": "User" }, "401": { "description": "Unauthorized" } }
      }
    },
    "/users": {
      "get": {
        "summary": "List users",
        "security": [{ "bearer": [] }],
        "responses": { "200": { "description": "Page of users" } }
      },
      "post": {
        "summary": "Create user (admin)",
        "security": [{ "bearer": [] }],
        "responses": { "201": { "description": "Created" }, "403": { "description": "Forbidden" } }
      }
    },
    "/ws": {
      "get": {
        "summary": "WebSocket echo",
        "responses": { "101": { "description": "Switching Protocols" } }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "bearer": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT" }
    }
  }
}
