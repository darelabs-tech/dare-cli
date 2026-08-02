# `dare ai`

Gerencia integrações diretas de IA, fornecendo diagnósticos de conexões, listagem de provedores ativos e ferramentas de enriquecimento semântico offline ou online para os artefatos de governança.

## Uso

```bash
dare ai <SUBCOMMAND> [OPTIONS]
```

## Subcomandos Disponíveis

| Subcomando | Descrição |
|---|---|
| `doctor` | Verifica a integridade e disponibilidade de executáveis de IA na PATH |
| `providers` | Lista todos os provedores e aliases de LLM configurados |
| `run` | Enriquece seções específicas dos artefatos de documentação metodológica |
| `prompt` | Renderiza ou visualiza o prompt gerado para uma seção sem enviar ao provedor |

---

## Verificação com `dare ai doctor`

O subcomando `doctor` executa verificações de disponibilidade rápidas e leves para os programas configurados de IA (sem rodar chamadas complexas com `--help` para evitar travamento de timeout):

| Status retornado | Significado |
|---|---|
| `ready` | O executável do provedor foi localizado e está pronto |
| `missing` | O executável correspondente ao provedor não foi encontrado na PATH ou o executável de override está ausente |
| `invalid` | A configuração de override de parâmetros está vazia ou malformada |
| `not_implemented` | O provedor é conhecido pela CLI DARE, mas a funcionalidade de enriquecimento ainda não foi implementada na versão atual |

### Provedores de IA Suportados (`ProviderId`)

| Provedor | Nome do Binário | Override por Env |
|---|---|---|
| `mock` | (embutido) | (Sempre `ready`, usado em CI/testes) |
| `codex` | `codex` (padrão) | `DARE_CODEX_COMMAND` |
| `claude-code` | `claude-code` | `DARE_CLAUDE_COMMAND` (Stub no Alpha) |
| `cursor-cli` | `cursor-cli` | `DARE_CURSOR_COMMAND` (Stub no Alpha) |
| `antigravity-cli` | `antigravity-cli` | `DARE_ANTIGRAVITY_COMMAND` (Stub no Alpha) |

---

## Enriquecimento de Seções (`dare ai run`)

Permite injetar contexto e descrições refinadas por IA nas seções de marcação `<!-- AGENT:BEGIN/END -->` declaradas no `DESIGN.md` ou `BLUEPRINT.md`.

- **Política de Gravação Segura (No-Write default):** Por padrão, rodar `dare ai run` apenas exibe o texto enriquecido na tela ou retorna em formato JSON estruturado. Ele **não** altera o arquivo fisicamente.
- **Gravação Explícita (`--write`):** A alteração física do arquivo só ocorre caso o usuário passe a flag `--write` em conjunto com a flag `--markdown <caminho>` explicitamente. O processo realiza validação do schema antes de escrever atomicamente no disco.

---

## Exemplos de Uso

```bash
# Executa diagnóstico dos provedores instalados
dare ai doctor

# Lista todos os provedores configurados
dare ai providers

# Enriquece a seção de descrição no DESIGN.md, mas apenas exibe o resultado na tela (no-write)
dare ai run design --section description

# Aplica fisicamente o enriquecimento da IA no arquivo do projeto
dare ai run design --section description --markdown DARE/DESIGN.md --write
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Sucesso — Diagnóstico concluído ou enriquecimento gerado |
| `2` | Uso de argumentos inválidos (ex.: chamar `--write` sem especificar `--markdown`) |
| `3` | O arquivo de markdown especificado não foi encontrado |
| `4` | Entrada inválida ou erro na validação do schema retornado pela IA |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
| `124` | Tempo limite (timeout) excedido durante a requisição de enriquecimento (`ENRICH_TIMEOUT = 20 minutos`) |
