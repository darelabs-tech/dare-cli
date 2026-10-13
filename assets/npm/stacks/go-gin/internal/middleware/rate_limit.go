package middleware

import (
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"golang.org/x/time/rate"
)

// RateLimit returns a token-bucket-per-IP middleware. Idle IP entries are
// pruned every 5 minutes to keep memory bounded.
func RateLimit(perSec, burst int) gin.HandlerFunc {
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

	return func(c *gin.Context) {
		ip := c.ClientIP()
		mu.Lock()
		b, ok := buckets[ip]
		if !ok {
			b = &bucket{limiter: rate.NewLimiter(rate.Limit(perSec), burst)}
			buckets[ip] = b
		}
		b.seen = time.Now()
		mu.Unlock()

		if !b.limiter.Allow() {
			c.AbortWithStatusJSON(http.StatusTooManyRequests, gin.H{"error": "rate limit exceeded"})
			return
		}
		c.Next()
	}
}
