# Persisted contracts (Ciclo 007)

Tipos e readers/writers canônicos em `dare-contracts`.

## Tipos ↔ paths

| Tipo | Path típico |
|------|-------------|
| `DareConfig` | `dare.config.json` |
| `DagV21` / `LegacyDag` | `DARE/dare-dag.yaml` |
| `RuntimeStateV1` | `.dare/state.json` |
| `GraphDocument` | `dare-graph.yml` |
| `SkillsManifest` | `.dare/skills.yml` |
| `VerificationBaseline` | `.dare/verification/*.json` |
| `UpdateManifestV1` | `templates/UPDATE-MANIFEST.json` |
| `TelemetrySnapshot` | in-memory / futuro |

## Política

| Tema | Decisão |
|------|---------|
| Flatten | Raiz + blocos tipados (`extra: Map`) — ADR-002 |
| Cap | 2 MiB (`MAX_CONTRACT_BYTES`) |
| JSON write | keys lexicográficas (`to_canonical_json_string`) |
| YAML | `yaml_serde` 0.10.4 as `serde_yaml`; igualdade semântica |
| Schema crate | `CONTRACTS_SCHEMA_VERSION = 0.1.0-contracts` |

## Security (RS-01…RS-09)

| ID | Status |
|----|--------|
| RS-01 path validation | ✅ |
| RS-02 fixtures sem secrets | ✅ |
| RS-03 ProjectRoot jail | ✅ |
| RS-04 audit/deny | ✅ (gate) |
| RS-05 sem secrets hardcoded | ✅ |
| RS-06 sem exec de YAML/JSON | ✅ |
| RS-07 size cap | ✅ |
| RS-08 atomic_write | ✅ |
| RS-09 sem Command | ✅ |

## Release notes — Ciclo 007

- Contratos persistidos + fixtures round-trip
- Pin: `yaml_serde 0.10.4` (import `serde_yaml`)
- Ver DEC-008

## Referências

- DESIGN/BLUEPRINT-007 · DEC-008 em [`DECISION-LOG.md`](../DECISION-LOG.md)
