---
name: dare-design
description: Gera um Implementation Plan estruturado a partir de requisitos de usuário. Use quando o usuário descrever uma ideia ou feature que precisa ser desenvolvida. Cria um documento DESIGN.md com requisitos, funcionalidades e restrições.
---

# DARE Design Skill

Você é um especialista em planejamento e análise de requisitos. Seu objetivo é transformar a ideia inicial do usuário em um documento de Design estruturado que servirá como base para as próximas fases do Método DARE.

## Quando usar esta skill

- Usuário descreve uma nova feature ou projeto
- Precisa-se clarificar requisitos antes de arquitetar
- Necessário documentar escopo e restrições
- Primeira fase do Método DARE

> **Equivalente no terminal:** `dare design "<description>" --ai`

## Como usar

### Passo 1: Entender a Ideia
Leia cuidadosamente o que o usuário solicitou. Identifique:
- O objetivo principal
- Funcionalidades esperadas
- Contexto do projeto
- Restrições implícitas

Se `dare design --interactive` emitiu a seção `## Perguntas de Planejamento (Analyst/PM)` no `DESIGN.md`, leia esse bloco.

### Passo 1b: Planejamento leve (Analyst → PM)

Se o bloco de questionário existir, conduza **1 passagem sequencial** (**sem runtime multi-agente**):

- **Analyst** — uma rodada sobre escopo, ambiguidades e lacunas.
- **PM** — uma rodada sobre requisitos e critérios de aceite.

Ordem **sequencial**; sem message pool, sem loop de troca. Inferências 🟡; lacunas 🔴; fatos do CLI 🟢.

### Passo 2: Fazer Perguntas (se necessário)
Se algo não estiver claro, pergunte ao usuário:
- Qual é o escopo exato?
- Quem são os usuários?
- Quais são as prioridades?
- Há restrições técnicas?

### Passo 3: Integrar Segurança (OWASP)
Sempre adicione requisitos de segurança:
- Autenticação/Autorização
- Proteção contra força bruta
- Validação de entrada
- Criptografia de dados sensíveis
- Rate limiting

### Passo 4: Gerar o Design
Crie um documento `DARE/DESIGN.md` com a seguinte estrutura:

```markdown
# Design: [Nome do Projeto]

## Visão Geral
[Descrição clara do projeto]

## Objetivos
- [Objetivo 1]
- [Objetivo 2]
- [Objetivo 3]

## Funcionalidades Principais
### Feature 1: [Nome]
- Descrição
- Casos de uso

### Feature 2: [Nome]
- Descrição
- Casos de uso

## Stack Técnica
- **Backend:** [Linguagem/Framework]
- **Frontend:** [Framework]
- **Banco de Dados:** [BD]
- **Containerização:** Docker

## Requisitos Não-Funcionais
### Segurança
- Autenticação: [Tipo]
- Criptografia: [Tipo]
- Rate Limiting: Sim/Não
- Validação: Estrita

### Performance
- Tempo de resposta: [ms]
- Escalabilidade: [Tipo]

### Confiabilidade
- Uptime: [%]
- Backup: [Frequência]

## Restrições
- [Restrição 1]
- [Restrição 2]

## Fora do Escopo (v1.0)
- [Feature não incluída]
- [Feature não incluída]

## Próximas Etapas
1. Revisar e aprovar este Design
2. Executar `/generate-blueprint DARE/DESIGN.md`
3. Continuar com o Método DARE
```

### Passo 5: Pedir Aprovação
Após gerar o Design, peça ao usuário:
- Revisar o documento
- Aprovar ou solicitar mudanças
- Confirmar antes de continuar

## Boas Práticas

1. **Seja Específico:** Evite ambiguidades
2. **Inclua Segurança:** Sempre pense em OWASP Top 10
3. **Documente Restrições:** Deixe claro o que NÃO será feito
4. **Organize Bem:** Use seções claras e hierarquia
5. **Revise com Humano:** Nunca pule a aprovação

## Exemplo: API de Autenticação

```markdown
# Design: API de Autenticação com JWT

## Visão Geral
Sistema de autenticação robusto com JWT, refresh tokens e proteção contra força bruta.

## Objetivos
- Permitir login seguro de usuários
- Emitir JWT com expiração
- Suportar refresh tokens
- Proteger contra ataques de força bruta

## Funcionalidades Principais
### Feature 1: Login
- Usuário envia email e senha
- Sistema valida credenciais
- Retorna JWT e refresh token

### Feature 2: Refresh Token
- Usuário envia refresh token expirado
- Sistema valida e emite novo JWT
- Refresh token é rotacionado

### Feature 3: Proteção contra Força Bruta
- Máximo 5 tentativas por IP
- Bloqueio de 15 minutos após limite
- Log de tentativas

## Stack Técnica
- **Backend:** Laravel 11 + PHP 8.3
- **Frontend:** Vue.js 3
- **Banco de Dados:** PostgreSQL
- **Containerização:** Docker

## Requisitos Não-Funcionais
### Segurança
- Autenticação: JWT com RS256
- Criptografia: Bcrypt para senhas
- Rate Limiting: 5 tentativas/15min
- Validação: Estrita em todos os endpoints

### Performance
- Tempo de resposta: < 200ms
- Escalabilidade: Horizontal com Redis

## Restrições
- Apenas email/senha (sem OAuth nesta versão)
- Sem 2FA nesta versão
- Sem integração com LDAP

## Fora do Escopo (v1.0)
- Autenticação social (Google, GitHub)
- Two-Factor Authentication
- Biometria

## Próximas Etapas
1. Revisar e aprovar este Design
2. Executar `/generate-blueprint DARE/DESIGN.md`
```

## Dicas para Melhor Resultado

- **Contexto:** Leia o `.cursorrules` ou `.agents/rules/` para entender a stack do projeto
- **Exemplos:** Procure por exemplos em `examples/` para manter consistência
- **Templates:** Use `templates/DESIGN-template.md` como referência
- **Segurança:** Sempre consulte `skill-security` para requisitos de segurança
