// {{projectName}} — net/http stdlib API entrypoint.
package main

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"os"
	"strconv"

	"github.com/joho/godotenv"

	"{{moduleName}}/internal/config"
	"{{moduleName}}/internal/db"
	"{{moduleName}}/internal/handler"
	"{{moduleName}}/internal/middleware"
	"{{moduleName}}/internal/repository"
	"{{moduleName}}/internal/service"
)

func main() {
	_ = godotenv.Load()

	cfg, err := config.FromEnv()
	if err != nil {
		log.Fatalf("config: %v", err)
	}

	pool, err := db.Connect(context.Background(), cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("db: %v", err)
	}
	defer pool.Close()

	repo := repository.NewUsersRepository(pool)
	svcAuth := service.NewAuthService(repo, cfg)
	svcUsers := service.NewUsersService(repo, cfg)

	authH := handler.NewAuthHandler(svcAuth, svcUsers, cfg)
	usersH := handler.NewUsersHandler(svcUsers)
	wsH := handler.NewWSHandler()

	mux := http.NewServeMux()

	// Public routes
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	})
	mux.HandleFunc("GET /openapi.json", func(w http.ResponseWriter, _ *http.Request) {
		b, _ := os.ReadFile("openapi.json")
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(b)
	})
	mux.HandleFunc("POST /auth/login", authH.Login)

	// Bearer-protected routes
	jwtMW := middleware.JWT(cfg)
	mux.Handle("GET /auth/me", jwtMW(http.HandlerFunc(authH.Me)))
	mux.Handle("GET /users", jwtMW(http.HandlerFunc(usersH.List)))
	mux.Handle("POST /users", jwtMW(http.HandlerFunc(usersH.Create)))

	// WebSocket — upgrade endpoint
	mux.HandleFunc("GET /ws", wsH.Handle)

	// Global middleware chain
	handler := middleware.Chain(
		mux,
		middleware.CORS(cfg.CORSOrigins),
		middleware.RateLimit(cfg.RateLimitPerSec, cfg.RateLimitBurst),
	)

	addr := ":" + strconv.Itoa(cfg.AppPort)
	log.Printf("listening on %s", addr)
	srv := &http.Server{
		Addr:    addr,
		Handler: handler,
	}
	if err := srv.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}
