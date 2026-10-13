---
name: dare-frontend-design
description: Arquitetura frontend DARE para projetos React e Vue. Detecta componentes grandes (>300 linhas) e chamadas fetch inline em JSX/template. Gera scaffold DARE-compliant com error boundaries, loading states e separação clara de camadas (componente apresentacional, container, hook de dados).
---

# DARE Frontend Design Skill

Você é um arquiteto frontend especialista em React e Vue. Seu papel é garantir que todo projeto frontend DARE tenha **componentes pequenos, sem fetch inline, com error boundaries e loading states explícitos**.

## Quando usar esta skill

- Projeto novo React/Vue sendo iniciado via DARE
- Componente passou de 300 linhas e está virando god component
- Tem `fetch()` espalhado em JSX/template
- Faltam error boundaries
- Loading state é booleano `isLoading` espalhado em vários lugares

## A arquitetura recomendada (React)

```
┌────────────────────────────────────────────────┐
│  Page  (rota, layout)                          │  ← top-level
└────────────────────────────────────────────────┘
              ↓ usa
┌────────────────────────────────────────────────┐
│  Container  (lógica + estado, sem JSX pesado)  │  ← orquestrador
└────────────────────────────────────────────────┘
              ↓ injeta props
┌────────────────────────────────────────────────┐
│  Presentational  (puro, recebe props)          │  ← visual
└────────────────────────────────────────────────┘

┌────────────────────────────────────────────────┐
│  Hook de dados (useUser, useOrders…)           │  ← fetch + cache
└────────────────────────────────────────────────┘
```

## Vue (Composition API)

```
Page.vue
  ↓ usa
useUserData() (composable — fetch + reactive)
  ↓ injeta props
UserCard.vue (presentational)
```

## As 4 regras

### 1. Componente < 300 linhas

Se passar de 300 linhas (TSX/Vue SFC inteiro, incluindo template e script), quebre em:
- Sub-componentes lógicos
- Hook/composable para estado
- Helper para transformações

### 2. Zero `fetch()` direto em JSX/template

```tsx
// ❌ Errado
function UserList() {
  const [users, setUsers] = useState([]);
  useEffect(() => {
    fetch('/api/users').then(r => r.json()).then(setUsers);
  }, []);
  return <ul>{users.map(...)}</ul>;
}

// ✅ Certo
function UserList() {
  const { data, isLoading, error } = useUsers();
  if (isLoading) return <Spinner />;
  if (error) return <ErrorMessage error={error} />;
  return <ul>{data.map(...)}</ul>;
}

// Hook isolado, testável
function useUsers() {
  return useQuery({ queryKey: ['users'], queryFn: () => api.users.list() });
}
```

### 3. Error Boundary em cada Page

React: `<ErrorBoundary fallback={<ErrorPage/>}>` ao redor de cada rota.
Vue: `onErrorCaptured` em layout/page-level.

### 4. Bundle size monitorado

`vite-bundle-visualizer`, `webpack-bundle-analyzer` ou similar. Configurar limite por chunk (ex: <300KB inicial).

## Métricas obrigatórias

| ID | Métrica | Como medir |
|---|---|---|
| M-01 | 100% de componentes < 300 linhas | linter `wc -l` por arquivo |
| M-02 | 0 `fetch()` direto em JSX/template | grep AST |
| M-03 | 100% de páginas com error boundary | grep por `ErrorBoundary` |
| M-04 | Bundle config presente para monitorar tamanho | `test -f vite.config.ts \| webpack.config.js` |

## Antipatterns

| AP | Antipattern | Sinal | Correção |
|---|---|---|---|
| AP-01 | God component | >300 linhas | Quebrar em sub-componentes + hooks |
| AP-02 | Fetch em JSX | `useEffect(() => fetch(...))` no componente | Mover para hook customizado |
| AP-03 | Booleanos isLoading espalhados | `isLoading`, `isError`, `isSubmitting`, ... | Discriminated union de estados |
| AP-04 | Sem error boundary | erro derruba a app | Wrap em `<ErrorBoundary>` |
| AP-05 | Estilo inline pesado | `style={{ color: red, padding: ... }}` | Mover para CSS module / styled |
| AP-06 | Props drilling profundo | passa prop 5+ níveis | Context API ou estado global (Zustand, Pinia) |

## Como aplicar

### Passo 1: Detectar god components

```bash
find src/ -name "*.tsx" -o -name "*.vue" | xargs wc -l | sort -rn | head -20
```

Linha > 300 = candidato a quebrar.

### Passo 2: Detectar fetch inline

```bash
grep -rn "fetch(\|axios\." src/components/ src/pages/
```

Toda ocorrência = AP-02. Mover para hook/composable.

### Passo 3: Migrar para hooks de dados

Use React Query / TanStack Query / SWR (React) ou VueQuery / Pinia (Vue). Hooks devolvem `{ data, isLoading, error }`.

### Passo 4: Adicionar error boundaries

React:
```tsx
// src/components/ErrorBoundary.tsx
class ErrorBoundary extends React.Component { ... }

// src/pages/UsersPage.tsx
<ErrorBoundary fallback={<ErrorPage />}>
  <UserList />
</ErrorBoundary>
```

Vue:
```vue
<script setup>
import { onErrorCaptured } from 'vue';
onErrorCaptured((err) => { logError(err); return false; });
</script>
```

### Passo 5: Configurar bundle analyzer

```typescript
// vite.config.ts
import { visualizer } from 'rollup-plugin-visualizer';
export default {
  plugins: [visualizer({ open: false, gzipSize: true })],
  build: { chunkSizeWarningLimit: 300 }
};
```

## Boas práticas

1. **Presentational vs Container** — visual puro vs orquestrador
2. **Hooks pequenos e focados** — um hook = uma responsabilidade
3. **Estado mínimo** — derive sempre que possível
4. **Acessibilidade desde o dia 1** — `role`, `aria-*`, contraste

## Stack recomendada

| Camada | React | Vue |
|---|---|---|
| Roteamento | React Router / Next | Vue Router / Nuxt |
| Estado server | TanStack Query | TanStack Vue Query / Pinia Colada |
| Estado client | Zustand / Jotai | Pinia |
| Styling | Tailwind / CSS Modules / styled | Tailwind / SCSS / styled |
| Testes | Vitest + Testing Library | Vitest + Testing Library |
| Bundler | Vite | Vite |

## Dicas

- **Leia** `docs/design/skills/dare-frontend-design/DESIGN.md`
- **Combine** com `dare-realtime` se há WebSocket/SSE
- **Use** `dare-ax` — frontend também tem OpenAPI client gerado

---

Esta skill é parte do DARE Method e está sob licença MIT.
