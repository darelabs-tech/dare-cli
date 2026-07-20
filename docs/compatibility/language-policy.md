# Política operacional de idioma (RF-08)

Regras fechadas para uso de idioma no rewrite Rust e na documentação de governança. Referenciada por CI-009 (classe C).

## Regras obrigatórias

1. **Docs de governança:** redigir em **pt-BR** (`docs/`, ADRs, matrizes, decision log, READMEs de compatibilidade).
2. **Código Rust novo (mensagens):** usar **en-US** como default para strings de usuário, erros, help e logs orientados ao operador.
3. **Strings PT existentes (Classe A):** preservar verbatim até ADR-003 ser **Accepted** e migration note publicada; normalização em massa sem ADR é proibida.
4. **Mistura no mesmo comando novo:** **proibida** — um comando ou subcomando não pode alternar PT/EN na mesma superfície de mensagens (help, erro, stdout).

## Escopo

| Superfície | Idioma canônico | Classe |
|------------|-----------------|--------|
| Comandos, flags, chaves JSON, IDs | EN (estável) | A (CI-002, CI-004) |
| Mensagens legadas preservadas | PT (até ADR-003) | A |
| Mensagens novas no rewrite Rust | EN | — |
| Documentação de governança | PT-BR | — |

## Exceções e breaking

- Alteração de idioma em string Classe A exige processo de breaking change (§ `breaking-change-process.md`) + ADR-003.
- Itens classe C (CI-009) permanecem bloqueados para normalização até ADR-003 + entrada no `DECISION-LOG`.

## Validação

- PR que introduz comando novo com mistura PT/EN na mesma superfície: **rejeitar** em review.
- PR que altera string PT legada sem ADR-003 Accepted: **rejeitar** (RS-03).
