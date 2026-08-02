# `dare welcome`

Exibe o banner de boas-vindas do DARE CLI e o guia de início rápido.

## Uso

```bash
dare welcome [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--no-banner` | Suprime o banner (mesmo efeito que `DARE_NO_BANNER=1`) |
| `--no-color` | Remove cores ANSI da saída |
| `--json` | Saída em JSON estruturado |

## Comportamento TTY

O banner animado só é exibido quando `stdout` é um TTY real. Em pipes, redirecionamentos ou CI, é suprimido automaticamente via detecção `IsTerminal`.

```bash
dare welcome              # exibe banner + quick-start (TTY)
dare welcome | cat        # sem banner (pipe detectado)
dare welcome --no-banner  # força supressão
```

## Variáveis de ambiente

| Variável | Valores aceitos | Efeito |
|---|---|---|
| `DARE_NO_BANNER` | `1`, `true`, `TRUE`, `yes`, `YES` | Suprime banner globalmente |
| `NO_COLOR` | qualquer valor | Remove cores ANSI |
| `DARE_NO_COLOR` | qualquer valor | Remove cores ANSI |

## Saída JSON (`--json`)

```json
{
  "schemaVersion": 1,
  "status": "ok",
  "banner_shown": false,
  "message": "DARE CLI v4.0.0\nDesign. Architect. Review. Execute.\n..."
}
```

## Quick-start exibido

O `dare welcome` mostra o fluxo recomendado:

```
1. dare init          → inicializa o projeto
2. dare design        → define requisitos
3. dare blueprint     → gera arquitetura
4. dare execute       → implementa com Ralph Loop
5. dare review        → valida a implementação
```

## Exit codes

| Código | Quando |
|---|---|
| `0` | Sucesso |
| `1` | Erro interno |

> **Nota:** `dare welcome` nunca menciona `dare new` — esse comando não existe no DARE CLI.
