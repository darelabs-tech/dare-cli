# `dare dashboard`

Inicia o painel local de telemetria e monitoramento de execução do DARE CLI em modo estrito de leitura (read-only) via browser.

## Uso

```bash
dare dashboard [OPTIONS]
```

## Flags e Opções

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--bind <IP>` | string | `127.0.0.1` | Endereço IP para escutar as conexões (pode ser sobreposto por `DARE_MCP_BIND`) |
| `--port <PORT>` | u16 | `4100` | Porta TCP para iniciar o servidor (pode ser sobreposto por `DARE_MCP_PORT`) |

---

## O Servidor de Painel e REST (`AppMode`)

O DARE possui uma engine interna baseada na crate **`dare-server`** (desenvolvido com Axum e Tokio) que pode rodar em dois modos:

1. **`AppMode::Dashboard` (Modo Padrão):**
   - Servidor HTTP estrito para visualização gráfica das métricas de execução.
   - Fornece endpoints somente de leitura (`read-only`), exibindo consumo de tokens, tempo gasto, status das tasks no DAG e histórico de correções.
   - Contém os arquivos da interface embarcados diretamente no binário Rust (via `rust-embed`).
2. **`AppMode::Rest` (Modo de Controle API):**
   - Habilitado pelo comando interno `dare server --protocol rest` (escuta por padrão na porta `3000`).
   - Habilita controle completo de mutação via requisições HTTP (como alterar o status das tasks de forma remota via chamadas `PUT`).

---

## Segurança e Bypass de Loopback

A autenticação das conexões com o servidor segue regras de segurança:

- **Loopback (Bypass):** Conexões originadas do mesmo host local (`127.0.0.1` ou `::1`) são **isentas** de validação de token para facilitar a experiência do usuário. O token é ignorado, a menos que seja enviado e não bata com a chave configurada (neste caso, retorna `401 Unauthorized`).
- **External (Non-Loopback):** Conexões de rede externa exigem obrigatoriamente autenticação via Header `Authorization: Bearer <TOKEN>`. A comparação do token é feita em tempo constante para mitigar ataques de temporização.

> A chave secreta é lida da variável de ambiente `DARE_MCP_TOKEN`. Caso esteja vazia, a CLI gera automaticamente um UUID v4 temporário na inicialização. Por segurança, o valor da chave secreta **não** é exibido nos logs padrão de inicialização, a menos que a variável de ambiente `DARE_MCP_LOG_TOKEN=1` esteja explícita.

---

## Limite de Requisições

- O servidor do painel rejeita payloads e corpos de requisições maiores que **1.048.576 bytes** (1MB), retornando instantaneamente o status HTTP `413 Payload Too Large`.

---

## Rotas de Grafo (Apenas no Modo REST)

No modo REST completo, o servidor expõe rotas HTTP para integração e busca contextual no grafo do GraphRAG:

- `POST /graph/locate`: Executa busca direta de nós no grafo.
- `POST /graph/traverse`: Executa travessia BFS a partir de nós semente limitando a profundidade.
- `POST /graph/map-requirement`: Mapeia e prioriza nós do tipo `requirement`.

---

## Exemplos de Uso

```bash
# Inicia o painel local na porta padrão 4100
dare dashboard

# Altera a porta de escuta do painel
dare dashboard --port 4500
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Servidor encerrado normalmente (graceful shutdown acionado via interrupção `Ctrl+C`) |
| `2` | Uso de argumentos inválidos |
| `4` | Porta já em uso ou configuração inválida de IP |
| `5` | Falha ao inicializar o servidor de rede ou carregar assets locais |
