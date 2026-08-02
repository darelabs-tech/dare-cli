# Instalação

O DARE CLI é distribuído como um binário nativo compilado em Rust — **sem Node.js, sem npm, sem dependências de runtime**.

---

## Instalação Automática (Recomendado)

### macOS, Linux e FreeBSD

```bash
curl -fsSL https://darelabs.tech/install | sh
```

O script detecta automaticamente a plataforma, baixa o binário correto e adiciona ao `PATH`.

### Windows PowerShell

```powershell
irm https://darelabs.tech/install.ps1 | iex
```

---

## Gerenciadores de Pacotes

### Homebrew (macOS / Linux)

```bash
brew install darelabs/tap/dare
```

### WinGet (Windows)

```powershell
winget install DareLabs.Dare
```

### Cargo (a partir do código-fonte)

```bash
cargo install dare-cli
```

---

## Download Manual

Faça o download do binário pré-compilado para sua plataforma em:

**[github.com/darelabs-tech/dare-cli/releases/latest](https://github.com/darelabs-tech/dare-cli/releases/latest)**

| Plataforma | Arquivo |
|---|---|
| macOS (Apple Silicon) | `dare-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `dare-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `dare-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `dare-aarch64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `dare-x86_64-pc-windows-msvc.zip` |

---

## Verificando a Instalação

```bash
dare --version
# dare 4.0.0

dare info
# Mostra status de integridade do ambiente DARE
```

---

## Canais de Release

O DARE CLI tem dois canais de distribuição:

| Canal | Comando | Estabilidade |
|---|---|---|
| **stable** | `dare self update --channel stable` | Produção |
| **beta** | `dare self update --channel beta` | Pré-release |

O canal padrão após a instalação é **beta**. Para usar stable:

```bash
dare self update --channel stable
```

---

## Migração do npm (v3 → v4)

Se você vinha usando `@dewtech/dare-cli` via npm:

```bash
# Remover instalação npm (opcional)
npm uninstall -g @dewtech/dare-cli

# Instalar CLI nativa
curl -fsSL https://darelabs.tech/install | sh
```

> O pacote npm `@dewtech/dare-cli@3.18.1` está em modo **legacy** — apenas correções de segurança até o fim da janela de suporte. Veja [docs/migration/npm-legacy-policy.md](https://github.com/darelabs-tech/dare-cli/blob/main/docs/migration/npm-legacy-policy.md).
