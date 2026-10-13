// Package httpx — small helpers around encoding/json for handlers.
package httpx

import (
	"encoding/json"
	"net/http"
)

// WriteJSON encodes v as JSON with the given status.
func WriteJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

// WriteError writes an RFC 7807 Problem Details JSON.
func WriteError(w http.ResponseWriter, status int, detail string) {
	WriteJSON(w, status, map[string]any{
		"type":   "urn:problem:" + http.StatusText(status),
		"title":  http.StatusText(status),
		"status": status,
		"detail": detail,
	})
}

// DecodeJSON reads a JSON body into v. Returns nil on success, error on
// malformed body or unexpected extra fields.
func DecodeJSON(r *http.Request, v any) error {
	dec := json.NewDecoder(r.Body)
	dec.DisallowUnknownFields()
	return dec.Decode(v)
}
