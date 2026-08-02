# `dare self`

Gerencia a instalação do próprio DARE CLI, permitindo atualizar o executável binário para novas versões, realizar rollbacks ou desinstalar de forma segura.

## Uso

```bash
dare self <ACTION> [OPTIONS]
```

## Ações Disponíveis

| Ação | Descrição |
|---|---|
| `update` | Atualiza o binário atual para a versão mais recente do canal correspondente |
| `rollback` | Restaura a versão imediatamente anterior mantida no backup local |
| `uninstall` | Remove o executável binário da máquina do usuário |
| `version` | Exibe a versão atual instalada do DARE CLI |

---

## Atualização Segura (`dare self update`)

O DARE CLI executa um pipeline robusto com verificação de integridade a cada atualização para evitar corrupção do ambiente:

```
1. Adquire lock exclusivo (~/.dare/self/update.lock)
2. Resolve a release no GitHub API (https://api.github.com)
3. Se --dry-run: exibe o plano e libera lock
4. Download do asset do binário + SHA256SUMS + SHA256SUMS.sig (temp dir)
5. Verifica SHA-256 do binário contra o SHA256SUMS
6. Verifica assinatura com cosign verify-blob (falha se "signing skipped")
7. Extrai o novo binário
8. Realiza backup do binário atual em ~/.dare/self/backup/
9. Substitui o executável atomicamente
10. Executa teste rápido de integridade (--version) no novo binário
    (Caso falhe, aciona o rollback automático restaurando o backup)
11. Libera o lock e remove arquivos temporários
```

- **Canais de Distribuição:**
  - **`beta` (padrão):** Canal de desenvolvimento ativo. Resolve para a release classificada como `prerelease: true` no GitHub (ou tags contendo `alpha`/`beta`).
  - **`stable`:** Canal de produção. Resolve para a release classificada como `prerelease: false`. Caso não exista nenhuma release estável na API, retorna o erro `stable channel has no non-prerelease GitHub Release (exit code 4)`.
- **Modo Offline e Assinaturas:** O processo exige assinatura digital ativa via **Cosign** (Keyless ou via chaves públicas oficiais). Se o `cosign` não for encontrado na PATH, o processo falha por padrão (`MSG_COSIGN_MISSING`). Para desenvolvimento local ou testes sem conexão, a verificação pode ser desativada definindo a variável de ambiente `DARE_SELF_ALLOW_UNSIGNED=1`.

---

## Rollback (`dare self rollback`)

Caso ocorra qualquer problema de estabilidade após uma atualização, o usuário pode reverter o processo de forma instantânea:
- Restaura o executável binário mantido em `~/.dare/self/backup/dare` para a localização padrão atual do executável do sistema.
- Caso o diretório de backup esteja vazio ou o arquivo tenha sido deletado, falha com `not_found (3)` (`MSG_NO_BACKUP`).

---

## Desinstalação (`dare self uninstall`)

O subcomando `uninstall` apaga o executável binário atual da máquina do usuário (`current_exe()`).
- Por segurança e para evitar a perda indesejada de projetos ativos, o DARE **não** apaga os metadados metodológicos, banco de dados locais de telemetria ou o diretório home global de cache (`~/.dare/self/`) durante a desinstalação padrão, a menos que sinalizado especificamente.

---

## Variáveis de Ambiente de Controle

| Variável | Padrão | Descrição |
|---|---|---|
| `DARE_SELF_HOME` | `~/.dare/self/` | Pasta base para armazenamento de backups, travas de segurança e canais |
| `DARE_SELF_RELEASE_API` | `https://api.github.com` | URL base para consulta de releases oficiais |
| `DARE_SELF_TIMEOUT_SECS` | `120` | Tempo limite em segundos para o download dos binários |
| `DARE_SELF_ALLOW_UNSIGNED` | (unset) | Permite pular a verificação de assinaturas digitais quando definido como `1` ou `true` |

---

## Exemplos de Uso

```bash
# Consulta atualizações disponíveis sem gravar nada na máquina
dare self update --dry-run

# Atualiza a CLI para a versão mais recente do canal de produção (stable)
dare self update --channel stable

# Atualiza para uma versão específica
dare self update --version 4.0.0

# Restaura a versão anterior mantida no backup
dare self rollback

# Desinstala a CLI do sistema
dare self uninstall
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Sucesso — Atualização, rollback ou desinstalação concluída com sucesso |
| `2` | Uso de argumentos inválidos |
| `3` | Não foi possível resolver a localização atual do binário no disco, ou backup não encontrado |
| `4` | Entrada inválida (como confirmação negada, lock de atualização ocupado ou canal de produção vazio) |
| `5` | Falha de conexão de rede, HTTP não-2xx ou timeout excedido no download |
| `6` | Falha de integridade: assinatura digital Ed25519 inválida, verificação cosign falhou ou mismatch no hash SHA256 |
