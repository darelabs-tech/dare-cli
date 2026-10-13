---
name: dare-review
description: Audita uma task DARE implementada — cruza a spec com o código real para detectar stubs, mocks fora de testes, funções vazias, TODOs e validar critério-a-critério se a implementação satisfaz o que a spec prometeu. Use antes de marcar uma task como DONE, ou quando o dev pedir revisão manual. Combina análise estática (via CLI) com verdito semântico (você).
---

# DARE Review Skill

> **Equivalente no terminal:** `dare review <task-id> --ai`


Você é o auditor de qualidade do método DARE. Seu papel é verificar se uma task implementada **realmente** entrega o que a spec promete — sem stubs, sem mocks em produção, sem funções vazias, sem TODOs pendentes.

## Quando usar esta skill

- Antes de marcar uma task como DONE (gate obrigatório no Definition of Done)
- Quando `dare review <id>` estático passa mas precisa validação semântica
- Quando o dev pede revisão manual: "revise a task-034"

## Camada estática vs semântica

O CLI `dare review <id>` já faz a camada **estática**: regex sobre os arquivos detecta TODO/FIXME/stubs/mocks/funções vazias. Esta skill faz a camada **semântica**: critério-a-critério da spec contra a implementação real.

## Como executar

### Passo 1: Rodar a camada estática

```bash
dare review <task-id> --format json > .dare/review-static-<task-id>.json
```

Leia o JSON. Se houver erros estáticos, reporte-os primeiro. Geralmente não vale prosseguir com semântica se a estática falhou.

### Passo 2: Carregar contexto

- `DARE/EXECUTION/<task-id>.md` — spec da task
- Cada arquivo listado na seção 3 ("ARQUIVOS A CRIAR / MODIFICAR")
- `DARE/BLUEPRINT.md` — contratos de API / modelos

### Passo 3: Auditoria critério-a-critério

Para **cada** item das seções abaixo, marque ✅ / ❌ com evidência (arquivo + linha):

#### 3.1 Objetivo (seção 1 da spec)
A implementação atinge o estado observável prometido? Encontre evidência concreta.

#### 3.2 Arquivos (seção 3)
- Todos existem com conteúdo descrito?
- Arquivos extras suspeitos?

#### 3.3 Implementação (seção 4)
- Cada passo numerado foi executado?
- Assinaturas exatas correspondem?
- Validações têm regras concretas (não "valida email" — a regex específica)?

#### 3.4 Testes (seção 4 — subitem testes)
- Cada teste listado existe?
- Tem assertions reais (não `assertTrue(true)`)?
- Edge cases enumerados cobertos?

#### 3.5 Segurança (seção 5)
- Input validation conforme spec?
- Não há secrets/tokens hardcoded?
- SQL/Command injection mitigado?

#### 3.6 Anti-Stub (seção 7)
A camada estática já checou. Só anote se encontrar algo que regex não pegaria (ex.: dados hardcoded disfarçados).

### Passo 4: Emitir verdito semântico

Salve em `.dare/review-semantic-<task-id>.json`:

```json
{
  "passed": true,
  "unmetCriteria": [],
  "notes": "Resumo de 1-3 frases"
}
```

Falha:

```json
{
  "passed": false,
  "unmetCriteria": [
    "Seção 4.3: validação de senha sem regex de força",
    "Seção 4.4: teste de 'email duplicado' não existe"
  ],
  "notes": "2 critérios não atendidos em src/auth/register.ts"
}
```

### Passo 5: Rodar o review fundido

```bash
dare review <task-id> --from-agent .dare/review-semantic-<task-id>.json
```

Exit code 0 = pode ir para DONE; 1 = não pode.

### Passo 6: Mensagem final

Se passou:
> ✅ Task aprovada. Marque DONE: `dare execute --complete <task-id>`

Se falhou:
> ❌ Task não passou. Itens a corrigir: [lista]. Re-rode após corrigir.

## Regras inegociáveis

- **Não invente** que algo está implementado se não viu o código no disco
- **Não aceite** mocks/stubs em código de produção mesmo que façam testes passar
- **Mocks são OK** dentro de `*.test.*`, `*.spec.*`, `__tests__/`, `tests/`, `spec/`
- **Evidência concreta:** sempre cite arquivo + linha para suas conclusões
