# Baseline de compatibilidade — @dewtech/dare-cli 3.18.1

Baseline oficial do pacote npm legado usada como referência de paridade no microplano 001.

## Verificação

Execute o comando canônico documentado no manifesto:

```bash
node scripts/governance/verify-baseline.mjs
```

Saída esperada em sucesso:

```json
{"ok":true,"package_version":"3.18.1","content_hash":"991121297f89c8360f865e90baba7586eb71c93eb2f3216b63453d16c76ce5af","matched":true}
```

O hash SHA-256 do tarball (bytes do `.tgz`, não do conteúdo extraído) deve coincidir com `content_hash` em `baseline-manifest.json`.

Para CI offline, defina `GOVERNANCE_TARBALL_PATH` apontando para um `.tgz` local de `@dewtech/dare-cli@3.18.1`.

## Fonte

- Pacote: `@dewtech/dare-cli@3.18.1`
- Registry: https://registry.npmjs.org/@dewtech/dare-cli/-/dare-cli-3.18.1.tgz
- SHA-256: `991121297f89c8360f865e90baba7586eb71c93eb2f3216b63453d16c76ce5af`
- Registrado em: 2026-07-20 por dare-labs

## Reprodução do hash

```bash
npm pack @dewtech/dare-cli@3.18.1 --pack-destination /tmp
# Linux/macOS:
shasum -a 256 dewtech-dare-cli-3.18.1.tgz
# Windows PowerShell:
Get-FileHash dewtech-dare-cli-3.18.1.tgz -Algorithm SHA256
```
