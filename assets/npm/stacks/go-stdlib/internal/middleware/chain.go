// Package middleware — composable HTTP middlewares for net/http.
package middleware

import "net/http"

// Middleware is the canonical wrapping shape.
type Middleware func(http.Handler) http.Handler

// Chain applies middlewares in order: Chain(h, A, B, C) → A(B(C(h))).
func Chain(h http.Handler, mws ...Middleware) http.Handler {
	for i := len(mws) - 1; i >= 0; i-- {
		h = mws[i](h)
	}
	return h
}

// contextKey is unexported so callers can't collide on the value.
type contextKey string

// ClaimsContextKey is the request-context key where JWT claims are stashed.
const ClaimsContextKey contextKey = "jwt.claims"
