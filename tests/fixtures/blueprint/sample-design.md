# DESIGN: Sample API Service

> **Versão:** v1.0 | **Data:** 1970-01-01 | **Status:** DRAFT

---

## 1. DESCRIÇÃO

A minimal sample design for blueprint bundle generation tests. The service exposes a REST API
for managing widgets with authentication and audit logging.

---

## 4. REQUISITOS FUNCIONAIS

| ID | Requisito | Prioridade | Critério de aceite |
|----|-----------|------------|--------------------|
| RF-01 | Expose REST API for widgets | MUST | CRUD endpoints return expected status codes |
| RF-02 | Authenticate users via JWT | MUST | Invalid token returns 401 |
| RF-03 | Export metrics endpoint | SHOULD | `/metrics` returns Prometheus format |

> Prioridades: **MUST** (bloqueia v1) · **SHOULD** (importante, mas não bloqueia) · **COULD** (nice to have)

---

## 5. REQUISITOS NÃO-FUNCIONAIS

| ID | Categoria | Requisito | Meta |
|----|-----------|-----------|------|
| RNF-01 | Performance | p95 latency under 200ms | Load test |

---

## 7. STACK TÉCNICA

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| Linguagem | Rust | 1.85 |
| Framework | Axum | 0.8 |
