# Política de disco e JSON (RF-09)

Regras fechadas para schemas persistidos, paths internos e contrato JSON de saída. Espelha Doc Mestre §13.3 e suporta CI-003 (classe A) e CI-008 (classe C → ADR-002).

## Tabela de políticas

| Tipo de mudança | Política |
|-----------------|----------|
| Leitura de arquivo legado | Obrigatória enquanto suportado |
| Escrita no formato legado | Manter até ADR autorizar nova versão |
| Novo campo opcional | Permitido com default seguro |
| Remoção/renomeação | Somente com migration + changelog |
| Alteração de ID canônico | Proibida sem migração integral |
| Alteração de exit code | Breaking change |
| Paths internos | Normalizar `/`; conversão correta no Windows |
| Ordenação | Determinística, independente de locale |
| Writers JSON/YAML | Canônicos; não depender de formatação acidental |

## Schemas persistidos (Classe A — CI-003)

Artefatos cobertos: `dare.config.json`, state stores (`.dare/state.json`), DAG YAML (`dare-dag.yaml`), manifests de baseline.

- **Leitura:** o rewrite deve ler formatos legados enquanto a baseline 3.18.1 os expõe.
- **Escrita:** persistir no formato legado até ADR autorizar bump de versão de schema.
- **Campos novos:** apenas opcionais, com default que não altere comportamento observável.
- **Remoção/renomeação:** exige migration script documentada + entrada no CHANGELOG + classificação na matriz (CI-008 → ADR-002).

## Saída JSON (`--json`)

- Ordenação de chaves **determinística**, independente de locale do SO.
- Campos e tipos estáveis documentados em ADR-002; divergências tratadas como CI-008 (`adr_required`).
- Writers devem emitir JSON/YAML canônico — sem depender de whitespace ou ordem acidental do runtime legado.

## Paths e exit codes

- Paths internos normalizados com separador `/` na lógica; conversão para paths nativos do Windows na camada de I/O.
- Alteração de exit code público documentado na baseline: **breaking change** — seguir `breaking-change-process.md`.

## Segurança (Classe D)

Alterações de I/O que afetem path escape, symlink abuse ou zip-slip (CI-010, CI-013) corrigem imediatamente (`must_fix`) sem preservar comportamento inseguro legado.
