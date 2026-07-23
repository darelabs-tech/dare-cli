package main

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

type User struct {
	ID string
}

func main() {
	r := gin.Default()
	r.GET("/users", func(c *gin.Context) {})
	r.POST("/users", func(c *gin.Context) {})
	http.HandleFunc("/legacy", func(w http.ResponseWriter, r *http.Request) {})
}
