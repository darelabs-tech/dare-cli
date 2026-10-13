package handler

import (
	"context"
	"net/http"
	"time"

	"github.com/coder/websocket"
)

type WSHandler struct{}

func NewWSHandler() *WSHandler { return &WSHandler{} }

func (h *WSHandler) Handle(w http.ResponseWriter, r *http.Request) {
	c, err := websocket.Accept(w, r, &websocket.AcceptOptions{
		InsecureSkipVerify: true, // tighten for production
	})
	if err != nil {
		return
	}
	defer c.CloseNow()

	ctx, cancel := context.WithTimeout(r.Context(), 10*time.Minute)
	defer cancel()

	for {
		typ, data, err := c.Read(ctx)
		if err != nil {
			return
		}
		if err := c.Write(ctx, typ, data); err != nil {
			return
		}
	}
}
