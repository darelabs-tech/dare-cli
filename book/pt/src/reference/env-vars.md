# Variáveis de Ambiente

O DARE CLI consome variáveis de ambiente para modificar comportamentos em pipelines de integração contínua (CI), configurações de depuração locais ou overrides de segurança de execução.

---

## Lista Completa de Variáveis de Controle

| Variável | Tipo / Padrão | Descrição |
|---|---|---|
| `DARE_NO_BANNER` | bool (`0`) | Suprime a exibição animada do banner de boas-vindas do DARE em comandos interativos se definido como `1`, `true` ou `yes`. |
| `NO_COLOR` / `DARE_NO_COLOR` | bool | Desativa a renderização de cores ANSI nas saídas de texto e tabelas do console. |
| `DARE_LOG` | string (`warn`) | Nível mínimo de depuração e saída de logs. Valores válidos: `error`, `warn`, `info`, `debug`, `trace`. |
| `DARE_DIR` | path (`.`) | Diretório raiz alternativo do projeto DARE sob o qual o runtime irá operar (sobrescreve o cwd). |
| `DARE_CHANNEL` | string (`beta`) | Canal padrão de verificação de releases do `dare self update` (valores: `stable`, `beta`). |
| `DARE_MCP_PORT` | u16 (`3777`) | Porta TCP padrão do servidor MCP HTTP (`AppMode::Rest` escuta em `3000` por padrão). |
| `DARE_MCP_BIND` | string (`127.0.0.1`) | Endereço IP de escuta para o servidor REST e MCP. |
| `DARE_MCP_TOKEN` | string | Token Bearer padrão exigido para autenticação em requisições de redes externas no servidor Axum. |
| `DARE_MCP_LOG_TOKEN` | bool (`0`) | Se definido como `1`, `true` ou `yes`, imprime o valor exato do token Bearer gerado no console de inicialização do servidor. |
| `DARE_MCP_BODY_LIMIT` | u32 (`1048576`) | Limite em bytes para payloads e dados de requisições no servidor. |
| `DARE_PROJECT_PATH` | path | Define o caminho absoluto para o diretório do projeto DARE que o servidor irá carregar. |
| `DARE_GUARD_SCAN_RULES_PATH` | path | Caminho para um arquivo JSON customizado contendo regras de auditoria e regex para o `dare guard`. |
| `DARE_GUARD_PRIVATE_KEY` | string | Chave privada Ed25519 em formato hexadecimal de 64 caracteres utilizada para assinar artefatos de controle. |
| `DARE_GUARD_PUBLIC_KEY` | string | Chave pública Ed25519 em formato hexadecimal utilizada para validar a assinatura dos artefatos metodológicos. |
| `DARE_SKILL_PRIVATE_KEY` | string | Chave privada Ed25519 em formato hexadecimal utilizada para assinar arquivos tar.gz de skills durante o `dare skill publish`. |
| `DARE_SELF_HOME` | path (`~/.dare/self/`) | Pasta global do sistema para armazenamento de backups, travas de escrita e históricos do binário. |
| `DARE_SELF_TIMEOUT_SECS` | u32 (`120`) | Tempo limite para downloads de ativos durante o `dare self update`. |
| `DARE_SELF_RELEASE_API` | URL | URL base da API para consulta de releases (padrão: `https://api.github.com`). |
| `DARE_SELF_ALLOW_UNSIGNED` | bool (`0`) | Se definido como `1` ou `true`, permite atualizar o executável da CLI no `self update` pulando a validação de assinatura do Cosign. |
