# Processo de breaking change (RF-11)

Fluxo obrigatório antes de alterar contratos Classe A, comportamentos documentados na baseline 3.18.1 ou políticas fechadas deste pacote.

## Máquina de estados

```text
Proposed → ADR Draft → Review(Tech Lead) → Review(PO) → Accepted
         → Changelog entry + Migration note (se disco/ID/exit)
         → Merge permitido
```

| Estado | Responsável | Saída |
|--------|-------------|-------|
| **Proposed** | Autor da mudança | Issue ou rascunho descrevendo impacto e item CI |
| **ADR Draft** | Autor + Tech Lead | ADR com status `Proposed`, alternativas e consequências |
| **Review(Tech Lead)** | Tech Lead DARE CLI | Comentários técnicos; ADR ajustado ou bloqueado |
| **Review(PO)** | Product Owner | Aceite de impacto em usuários e semver |
| **Accepted** | Tech Lead | Frontmatter ADR `status: Accepted` + link na matriz |
| **Changelog + Migration** | Autor | Entrada CHANGELOG; migration note se disco, ID canônico ou exit code |
| **Merge permitido** | Maintainer | PR aprovado com checklist completo |

Merge **sem** ADR Accepted para mudança breaking = violação RS-03 — rejeitar em review humano.

## O que é breaking (lista fechada — ciclo 001)

1. Alteração de exit code documentado
2. Remoção/renomeação de flag ou comando público
3. Remoção/renomeação/mudança de tipo de campo JSON público ou schema de disco
4. Alteração de ID canônico
5. Substituição silenciosa REST↔MCP

Qualquer item acima exige ADR Accepted + atualização de `classification-matrix.md` antes do merge.

## Pré-condição de merge (PR checklist)

Antes de merge na branch principal, o PR deve marcar:

- [ ] **ADR Accepted linkado** no corpo do PR (referência `ADR-NNN`)
- [ ] **`classification-matrix.md` atualizada** com classe e ação corretas
- [ ] **`DECISION-LOG` entrada** quando aplicável (waivers, escopo deferido, exceções)

CI opcional (COULD): falhar se paths de contrato mudarem sem referência `ADR-` no PR — não bloqueia ciclo 001.

## Waiver Classe D (RS-07)

Itens CI-010..CI-014 (`must_fix`): correção imediata permitida **sem** preservar comportamento inseguro legado. Registrar na matriz; ADR recomendada mas não bloqueia hotfix de segurança.

## Artefatos relacionados

| Artefato | Papel |
|----------|-------|
| `classification-matrix.md` | Classificação A/B/C/D por item |
| `docs/adr/` | Decisões normativas |
| `docs/DECISION-LOG.md` | Waivers e deferrals (ex.: DEC-001) |
| `baseline-3.18.1.md` | Referência de paridade legado |
