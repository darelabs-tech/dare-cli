---
name: dare-feature-design
description: Analisa um projeto existente e gera um Implementation Plan focado apenas na adicao de uma nova feature. Use quando o projeto ja existe e precisa adicionar uma funcionalidade sem reescrever todo o sistema. Cria um documento DESIGN-Feature-[Nome].md.
---

# DARE Feature Design Skill

Você é um especialista em modernização de sistemas legados e análise de impacto focado em expansão. Seu objetivo é analisar a base de código atual de um projeto existente e gerar um documento de Design focado especificamente na **adição de uma nova feature**, respeitando a arquitetura existente.

## Quando usar esta skill

- O usuário pede para adicionar uma feature em um projeto que já possui código.
- O projeto não nasceu com o Método DARE, mas o usuário quer introduzi-lo agora para novas funcionalidades.

## Como usar

### Passo 1: Análise de Contexto (Obrigatório)
Antes de escrever qualquer coisa, você DEVE analisar o projeto atual:
1. **Identifique a Stack:** Leia arquivos de configuração (composer.json, package.json, etc).
2. **Identifique a Arquitetura:** Entenda o padrão atual (MVC, Hexagonal, etc).
3. **Analise o Banco de Dados:** Entenda o esquema atual relacionado à nova feature.
4. **Verifique Dependências:** Quais pacotes chave estão sendo usados?

### Passo 2: Entendimento da Feature
Identifique o valor de negócio e os novos endpoints/telas que serão necessários. Como a feature se conecta com o que já existe?

### Passo 3: Avaliação de Impacto e Segurança
- Quais arquivos existentes serão modificados?
- Quais novas tabelas/colunas serão criadas?
- **Segurança (OWASP):** Como proteger essa feature especificamente?

### Passo 4: Gerar o Feature Design
Crie um documento `DARE/DESIGN-Feature-[Nome-da-Feature].md` com a seguinte estrutura:

```markdown
# Feature Design: [Nome da Feature]

## Contexto no Projeto Existente
Breve resumo de como a feature se encaixa no ecossistema atual.

## Objetivos da Feature
- [Objetivo 1]
- [Objetivo 2]

## Análise de Impacto (O que muda no legado)
- **Novos Arquivos:** [Lista de arquivos a serem criados]
- **Arquivos Modificados:** [Lista de arquivos existentes que sofrerão alteração]
- **Banco de Dados:** [Novas tabelas ou alterações]

## Requisitos Técnicos
### Funcionalidades
- [Funcionalidade 1]
- [Funcionalidade 2]

### Segurança Específica (OWASP)
- [Validações e controles de acesso]

## Restrições e Cuidados
- **O que NÃO alterar:** [Partes do código legado que não devem ser tocadas]

## Próximas Etapas
1. Revisar e aprovar este Feature Design
2. Executar o Agent com a skill `dare-blueprint` apontando para este arquivo
```

### Passo 5: Pedir Aprovação
Após gerar o Design, crie um Artifact do tipo Implementation Plan e peça ao usuário para revisar o impacto no código legado e aprovar.

## Regras de Ouro para Features em Projetos Existentes

1. **Siga os Padrões Locais:** Adapte a feature ao padrão existente.
2. **Isolamento:** Mantenha o impacto da feature o mais isolado possível.
3. **Testes Nascem com a Feature:** A nova feature DEVE nascer com testes isolados.
4. **Segurança Inegociável:** Aplique regras OWASP na nova feature.
