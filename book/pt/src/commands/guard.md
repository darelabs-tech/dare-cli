# `dare guard`

Gate de segurança e integridade estrita sobre o código-fonte e os artefatos metodológicos da pasta `DARE/`. Ele audita caracteres especiais (Unicode), vazamento de segredos, assinaturas digitais Ed25519 e regras de segurança contra OWASP Top 10.

## Uso

```bash
dare guard [TARGET] [OPTIONS]
```

## Flags de Seleção

O comando pode ser executado em três modos distintos:

| Flag | Descrição |
|---|---|
| `[TARGET]` | Executa o scan sobre um arquivo ou diretório específico (padrão se nenhum modo for selecionado) |
| `--staged` | Executa a auditoria apenas sobre os arquivos adicionados na área de staging do Git (`git diff --cached`) |
| `--all` | Executa o scan em todo o projeto, ignorando pastas de compilação ou dependências (como `.git/`, `target/`, `node_modules/`, `.dare/agent-worktrees/`) |

## Flags Adicionais

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--unicode <MODE>` | string | `block` | Como tratar caracteres Unicode não-ASCII encontrados. Valores: `block` (bloqueia e falha), `warn` (emite avisos), `allow` (ignora) |
| `--strict` | bool | `false` | Trata qualquer aviso (warning) como falha e altera o veredito para FAIL |
| `--json` | bool | `false` | Saída formatada em JSON estruturado |

---

## O que o Guard valida?

O pipeline de execução do `dare guard` realiza as seguintes auditorias:

### 1. Filtro de Unicode (`--unicode`)
Padrão de segurança contra ataques de homografia (Homograph attacks) ou injeção visual de scripts:
- Detecta caracteres Unicode fora do conjunto ASCII padrão.
- Se `--unicode block` (default), falha a validação caso encontre caracteres de alfabetos mistos ou suspeitos no código de produção.

### 2. Scanner de Regras (Secrets & OWASP)
Aplica expressões regulares descritas no arquivo `assets/rules/scan-rules.json` (ou embutidas) para procurar por:
- Vazamentos de chaves de API, senhas, tokens de JWT ou segredos de infraestrutura.
- Padrões de risco clássicos como injeções SQL, XSS ou sanitizações inadequadas.
- Permite override do arquivo de regras via variável de ambiente `DARE_GUARD_SCAN_RULES_PATH`.

### 3. Verificação de Assinatura Digital (Proveniência)
Se a checagem de assinatura estiver ativada na configuração do projeto (`signing.enabled`):
- Arquivos importantes de controle (como `dare.config.json` e arquivos dentro de `DARE/`) devem possuir uma assinatura digital válida com extensão `.minisig`.
- A assinatura utiliza criptografia **Ed25519** de alta performance.
- Garante que a proveniência dos artefatos foi mantida e não houve alteração externa indesejada (fora do escopo autorizado do agente de IA).

---

## Preflight Agent Checks

Quando um agente de IA executa uma task com a flag `--agent`, o sistema executa o método `run_preflight` do `dare-guard` internamente **antes** de iniciar o loop.
- O preflight valida todos os artefatos de controle sob `DARE/` e o `dare.config.json`.
- Caso alguma regra de segurança seja violada, o preflight falha com `GuardFail`, interrompendo a execução antes do início da escrita de código.

---

## Exemplos de Uso

```bash
# Audita apenas as alterações adicionadas ao staging do Git
dare guard --staged

# Executa auditoria estrita em todos os arquivos de um subdiretório
dare guard src/auth/ --strict

# Audita todo o projeto em modo JSON
dare guard --all --json
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | PASS — O projeto passou em todas as regras do Guard |
| `1` | Erro interno do sistema |
| `2` | Uso de argumentos inválidos |
| `3` | O alvo (`TARGET`) especificado não foi encontrado |
| `4` | Entrada inválida ou erro na leitura do arquivo de regras |
| `5` | Falha inesperada de I/O na leitura de arquivos |
| **6** | **Guard FAIL** — O projeto violou regras críticas de segurança ou assinatura inválida |
