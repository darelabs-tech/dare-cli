---
name: dare-rust-leptos
description: Guia DARE para desenvolvimento Leptos (Rust + WASM) — decisão CSR vs Fullstack, idioms 0.7, antipatterns, tipos compartilhados server+WASM com cfg_attr, workspace misto WASM+nativo, templates de tasks.
---

# DARE Rust/Leptos Skill

Você é um especialista em Leptos 0.7+ (framework reativo Rust → WASM). Seu papel é orientar a decisão de variante (CSR vs Fullstack), aplicar idioms obrigatórios, evitar antipatterns clássicos e estruturar tipos compartilhados entre server e WASM.

## Quando usar

- Projeto novo Leptos sendo iniciado
- Decisão CSR vs Fullstack precisa ser tomada
- Workspace misto (Leptos + Axum + napi-rs) com problemas de target
- Tipos compartilhados quebrando entre server e WASM
- Effect/Resource/Action sendo mal usados

## 1. Decisão de variante: CSR vs Fullstack

| Critério | CSR (trunk) | Fullstack (cargo-leptos) |
|---|---|---|
| SEO necessário | ❌ | ✅ |
| Time-to-interactive crítico | ❌ | ✅ |
| Dashboard interno / admin | ✅ | ✅ |
| Backend Axum existente | indiferente | ✅ integração direta |
| Simplicidade de deploy | ✅ arquivos estáticos | ⚠️ binário Axum |
| Server functions (`#[server]`) | ❌ não existe | ✅ |

**Regra de ouro:**
- Atrás de login e SEO não importa → **CSR**
- Precisa SEO, carregamento inicial rápido, ou server functions → **Fullstack**
- Já tem Axum no monorepo → **Fullstack** (workspace unificado)

## 2. Ferramentas — nunca misturar

| Variante | Build | Dev server | Test |
|---|---|---|---|
| CSR | `trunk build --release` | `trunk serve` | `cargo test --workspace` |
| Fullstack | `cargo leptos build --release` | `cargo leptos watch` | `cargo test --workspace` |

> `cargo leptos test` **não existe**. Use `cargo test --workspace`.
> Não use `trunk` para fullstack nem `cargo leptos` para CSR.

## 3. Idioms obrigatórios Leptos 0.7

### Estado reativo

```rust
// Signals — só re-renderiza o que usa
let (count, set_count) = signal(0);
let doubled = move || count.get() * 2;  // derived (memo inline)

// Para estado compartilhado:
let count = RwSignal::new(0);
```

### Dados assíncronos

```rust
// ✅ Resource — declarativo, integra com Suspense
let user = Resource::new(|| user_id(), |id| async move { fetch_user(id).await });

view! {
    <Suspense fallback=|| view! { <p>"Loading..."</p> }>
        {move || user.get().map(|u| view! { <p>{u.name}</p> })}
    </Suspense>
}

// ❌ Effect que faz fetch (re-executa em todo render)
```

### Mutações

```rust
// ✅ Action — submits, forms, operações
let save = Action::new(|input: &String| {
    let input = input.clone();
    async move { api::save(input).await }
});

view! {
    <button on:click=move |_| save.dispatch("hello".to_string())>"Save"</button>
    <Show when=move || save.pending().get()><p>"Saving..."</p></Show>
}
```

### Listas e condicionais

```rust
view! {
    <Show when=move || logged_in.get() fallback=|| view! { <Login/> }>
        <Dashboard/>
    </Show>

    <For
        each=move || items.get()
        key=|item| item.id
        children=move |item| view! { <ItemRow item=item/> }
    />
}
```

### Server functions (fullstack only)

```rust
#[server(SaveUser, "/api")]
pub async fn save_user(name: String) -> Result<User, ServerFnError> {
    // Só roda no server (feature = "ssr")
    db::create_user(name).await.map_err(Into::into)
}

// No componente
let save = ServerAction::<SaveUser>::new();
```

## 4. Tipos compartilhados server + WASM

```rust
// crates/<projeto>-domain/src/lib.rs
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]  // só no server
#[derive(Clone, Debug, Serialize, Deserialize)]      // server + WASM
pub struct SecurityEvent {
    pub id: uuid::Uuid,
    pub attack_type: String,
    pub risk_score: f32,
    pub blocked: bool,
}
```

`Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.10", features = ["v4", "serde"] }

[dependencies.sqlx]
version = "0.8"
features = ["postgres", "runtime-tokio", "uuid"]
optional = true

[features]
ssr = ["dep:sqlx"]
```

## 5. Workspace misto (WASM + nativo)

```toml
# Cargo.toml do workspace
[workspace]
resolver = "2"
members = [
  "crates/<p>-core",       # lib nativa (x86)
  "crates/<p>-server",     # bin nativo — Axum
  "crates/<p>-web",        # bin+lib WASM — Leptos
  "crates/<p>-cli",        # bin nativo
]
```

```toml
# crates/<p>-web/Cargo.toml — features separam server de WASM
[features]
default = []
hydrate = ["leptos/hydrate"]
ssr = ["dep:axum", "dep:leptos_axum", "dep:tokio", "leptos/ssr"]
```

> **Antipattern crítico:** `[build] target = "wasm32-unknown-unknown"` no `.cargo/config.toml` raiz quebra todos os crates nativos. `cargo leptos` gerencia o target WASM internamente — não interfira.

## 6. Antipatterns

| Antipattern | Por quê | Alternativa |
|---|---|---|
| `wasm_bindgen` direto | Bypassa abstrações Leptos, frágil | APIs do Leptos (`web_sys` via feature) |
| `panic!` em componentes | Derruba o app sem `ErrorBoundary` | `Result<_, ServerFnError>` + `ErrorBoundary` |
| `Effect` que faz fetch | Re-executa a cada render | `Resource::new()` |
| `tokio::spawn` no client | `tokio` não existe no WASM | `spawn_local()` ou server function |
| `std::thread` no client | Não existe no WASM | Signals para paralelismo reativo |
| `cargo leptos test` | Comando inválido | `cargo test --workspace` |
| `[build] target` global | Quebra crates nativos | Sem target global; cargo-leptos gerencia |

## 7. Templates de tasks DARE

Cole no `DARE/dare-dag.yaml` após blueprint:

```yaml
- id: leptos-001
  title: "Workspace, AppShell e Router base"
  description: |
    Cargo workspace com resolver = "2".
    App component com leptos_router::Router.
    Layout base (header, main, footer). Rota "/" → HomePage.
    Ralph Loop: cargo leptos build --release + cargo test --workspace + cargo clippy
  depends_on: []

- id: leptos-002
  title: "Form com Action + server function"
  description: |
    #[server] fn para processar o form.
    ActionForm ou Action manual.
    Validar input no server (ServerFnError em erro).
    Pending state com Action::pending().
  depends_on: [leptos-001]

- id: leptos-003
  title: "Lista paginada com Resource + Suspense"
  description: |
    Resource recebendo página como signal.
    Envolver em Suspense com fallback loading.
    ErrorBoundary para erros de fetch.
    Paginação com For + key por ID.
  depends_on: [leptos-001]
```

## 8. Stack canônica

| Camada | Cliente (WASM) | Server (Fullstack) |
|---|---|---|
| Framework | Leptos 0.7+ | Leptos 0.7 + leptos_axum |
| Roteamento | leptos_router | (compartilhado) |
| Estado server | Resource + Server Functions | sqlx + Axum |
| Estilo | Tailwind | — |
| Build | cargo-leptos / trunk | cargo-leptos |

## Como aplicar

### Passo 1: Decidir variante

Use a tabela em §1. Documente decisão no DESIGN.md.

### Passo 2: Setup workspace

Aplique `dare-rust-workspace` para layout multi-crate. `<p>-domain` em separado para tipos compartilhados.

### Passo 3: Implementar componentes seguindo idioms §3

Resource para fetch, Action para mutação, Show/For para condicional/lista.

### Passo 4: Server functions (fullstack)

`#[server]` para qualquer comunicação client→server. Validação no server, ServerFnError em erro.

### Passo 5: Ralph Loop validation

```bash
cargo leptos build --release        # fullstack
# ou
trunk build --release               # CSR
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Dicas

- **Combine** com `dare-rust-workspace` para decisão de layout
- **Para SSR**, use `leptos_axum` (não axum direto)
- **Hidratação** — sempre `view!` em ambos lados, server gera HTML inicial

---

Esta skill é parte do DARE Method e está sob licença MIT.
