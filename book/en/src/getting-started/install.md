# Installation

DARE CLI is distributed as a native binary compiled in Rust — **no Node.js, no npm, no runtime dependencies**.

---

## Automatic Installation (Recommended)

### macOS, Linux, and FreeBSD

```bash
curl -fsSL https://darelabs.tech/install | sh
```

The script automatically detects the platform, downloads the correct binary, and adds it to your `PATH`.

### Windows PowerShell

```powershell
irm https://darelabs.tech/install.ps1 | iex
```

---

## Package Managers

### Homebrew (macOS / Linux)

```bash
brew install darelabs/tap/dare
```

### WinGet (Windows)

```powershell
winget install DareLabs.Dare
```

---

## Source Compilation (Cargo)

```bash
cargo install dare-cli
```
