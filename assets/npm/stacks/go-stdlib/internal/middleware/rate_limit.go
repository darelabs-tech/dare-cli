package middleware

import (
	"encoding/json"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"golang.org/x/time/rate"
)

// RateLimit is a token-bucket-per-IP middleware. Idle entries are pruned
// every 5 minutes to keep memory bounded.
func RateLimit(perSec, burst int) Middleware {
	type bucket struct {
		limiter *rate.Limiter
		seen    time.Time
	}
	var (
		mu      sync.Mutex
		buckets = make(map[string]*bucket)
	)

	go func() {
		t := time.NewTicker(5 * time.Minute)
		defer t.Stop()
		for now := range t.C {
			mu.Lock()
			for ip, b := range buckets {
				if now.Sub(b.seen) > 10*time.Minute {
					delete(buckets, ip)
				}
			}
			mu.Unlock()
		}
	}()

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip := clientIP(r)
			mu.Lock()
			b, ok := buckets[ip]
			if !ok {
				b = &bucket{limiter: rate.NewLimiter(rate.Limit(perSec), burst)}
				buckets[ip] = b
			}
			b.seen = time.Now()
			mu.Unlock()

			if !b.limiter.Allow() {
				w.Header().Set("Content-Type", "application/json; charset=utf-8")
				w.WriteHeader(http.StatusTooManyRequests)
				_ = json.NewEncoder(w).Encode(map[string]any{
					"type":   "urn:problem:TooManyRequests",
					"title":  http.StatusText(http.StatusTooManyRequests),
					"status": http.StatusTooManyRequests,
					"detail": "rate limit exceeded",
				})
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

func clientIP(r *http.Request) string {
	if xff := r.Header.Get("X-Forwarded-For"); xff != "" {
		// First IP in chain.
		if i := strings.Index(xff, ","); i >= 0 {
			return strings.TrimSpace(xff[:i])
		}
		return strings.TrimSpace(xff)
	}
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		return r.RemoteAddr
	}
	return host
}
