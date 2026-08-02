# Configuração (`dare.config.json`)

O arquivo `dare.config.json` na raiz do projeto é a fonte central de verdade sobre as definições de stack, toolchain e comportamento das ferramentas do DARE CLI no projeto atual.

## Estrutura do Schema (versão 1)

O JSON do arquivo `dare.config.json` utiliza notação **camelCase** congelada e estruturada da seguinte forma:

```json
{
  "schemaVersion": 1,
  "projectName": "meu-projeto",
  "stack": "rust",
  "toolchain": "stable",
  "frontend": null,
  "transport": null,
  "graphrag": {
    "backend": "sqlite",
    "semantic": {
      "enabled": false
    }
  },
  "signing": {
    "enabled": false,
    "publicKey": "hexadecimal-public-key"
  }
}
```

---

## Detalhamento dos Campos

| Campo | Tipo | Obrigatório | Descrição |
|---|---|---|---|
| `schemaVersion` | integer | Sim | Versão do schema de configuração (atual: `1`). Alterações de schema exigem ADR. |
| `projectName` | string | Sim | Nome de identificação do projeto. Deve bater com o padrão regex: `^[a-z][a-z0-9_-]{0,63}$`. |
| `stack` | string | Sim | Stack técnica de desenvolvimento. Valores válidos: `rust`, `python`, `node-ts`, `go`, `php-laravel-11`, `ruby-rails-8`, `nestjs`. |
| `toolchain` | string | Sim | Toolchain de compilador/interpretador configurado no DARE bootstrap (ex.: `stable`, `1.85.0`). |
| `frontend` | string? | Não | Framework frontend integrado (se houver). Valores: `react`, `vue` ou `null`. |
| `transport` | string? | Não | Método padrão do transporte do MCP. Valores: `stdio`, `sse`, `http` ou `null`. |
| `graphrag` | object | Sim | Configurações da engine local de contexto. |
| `graphrag.backend` | string | Sim | Tecnologia utilizada pelo grafo. Valores suportados: `sqlite`, `neo4j`. |
| `graphrag.semantic.enabled` | boolean | Sim | Habilita a geração e busca híbrida de embeddings em modo semântico (default: `false`). |
| `signing` | object | Não | Bloco de chaves para verificação de assinaturas Ed25519 de segurança de artefatos. |
| `signing.enabled` | boolean | Sim | Se `true`, exige que o `dare guard` valide assinaturas `.minisig` de arquivos importantes (como `dare.config.json` e `DARE/`). |
| `signing.publicKey` | string? | Não | Chave pública hexadecimal para auditorias de proveniência de arquivos de controle. |

---

## Regra de Conflito e Dual Naming (ADR-006 / Config 008)

- **Configuração Dual Naming:** Para compatibilidade com versões legadas da CLI baseada em TypeScript, a leitura do arquivo suporta o mapeamento alternativo do termo `ide` e `backend` indistintamente em tempo de execução para resolver o provedor, mantendo a consistência do schema em disco.
- **Mutações:** As mutações nos campos de configuração são aplicadas de forma atômica através da API `save_config` (que faz write-then-rename), garantindo integridade dos dados em caso de parada abrupta ou queda de energia.
