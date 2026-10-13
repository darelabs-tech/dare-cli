// {{projectName}} — Gin API entrypoint.
//
// @title           {{projectName}}
// @version         0.1.0
// @description     DARE-shaped Gin API.
// @BasePath        /
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
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

	r := gin.New()
	r.Use(gin.Recovery())
	r.Use(middleware.RateLimit(cfg.RateLimitPerSec, cfg.RateLimitBurst))
	r.Use(middleware.CORS(cfg.CORSOrigins))

	r.GET("/healthz", func(c *gin.Context) { c.JSON(http.StatusOK, gin.H{"status": "ok"}) })
	r.GET("/openapi.json", func(c *gin.Context) {
		b, _ := os.ReadFile("openapi.json")
		var v any
		_ = json.Unmarshal(b, &v)
		c.JSON(http.StatusOK, v)
	})

	auth := handler.NewAuthHandler(svcAuth, svcUsers, cfg)
	r.POST("/auth/login", auth.Login)

	protected := r.Group("/", middleware.JWT(cfg))
	{
		protected.GET("/auth/me", auth.Me)
		users := handler.NewUsersHandler(svcUsers)
		protected.GET("/users", users.List)
		protected.POST("/users", users.Create)
	}

	ws := handler.NewWSHandler()
	r.GET("/ws", ws.Handle)

	addr := fmt.Sprintf(":%d", cfg.AppPort)
	log.Printf("listening on %s", addr)
	if err := r.Run(addr); err != nil {
		log.Fatal(err)
	}
}
