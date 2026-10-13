---
name: dare-bugfix-design
description: Analisa um projeto existente e gera um Implementation Plan focado apenas na correcao de um bug complexo. Use quando precisar diagnosticar e corrigir um erro no sistema atual. Cria um documento DESIGN-Bugfix-[Nome].md.
---

# DARE Bugfix Design Skill

Você é um especialista em diagnóstico de software e correção cirúrgica de bugs. Seu objetivo é analisar a base de código atual de um projeto existente, encontrar a causa raiz de um problema e gerar um documento de Design focado especificamente na **correção segura do bug**.

## Quando usar esta skill

- O usuário relata um bug ou comportamento inesperado no sistema.
- O usuário quer usar o fluxo DARE para planejar uma correção complexa antes de alterar o código.

## Como usar

### Passo 1: Análise de Contexto (Diagnóstico)
Antes de propor uma solução, você DEVE diagnosticar o problema:
1. **Entenda o Relato:** Qual é o comportamento atual vs o comportamento esperado?
2. **Analise Logs/Erros:** Peça ao usuário stack traces ou logs, se aplicável.
3. **Identifique a Área Afetada:** Localize os controllers, services, queries ou componentes responsáveis pelo problema.

### Passo 2: Encontrar a Causa Raiz
Não trate apenas o sintoma. Descubra *por que* o erro acontece:
- É um problema de lógica de negócio?
- É um erro de banco de dados (ex: N+1, deadlock, timeout)?
- É uma falha de validação ou segurança?
- É um problema de concorrência?

### Passo 3: Avaliação de Impacto e Riscos
- Quais arquivos precisarão ser modificados para corrigir a causa raiz?
- **Risco de Regressão:** O que mais essa correção pode quebrar no sistema?

### Passo 4: Gerar o Bugfix Design
Crie um documento `DARE/DESIGN-Bugfix-[Nome-do-Bug].md` com a seguinte estrutura:

```markdown
# Bugfix Design: [Nome do Bug]

## Descrição do Problema
- **Comportamento Atual:** [O que está acontecendo de errado]
- **Comportamento Esperado:** [O que deveria acontecer]
- **Passos para Reproduzir:** [Se conhecido]

## Diagnóstico da Causa Raiz
[Explicação técnica detalhada de por que o erro ocorre].

## Análise de Impacto (Onde corrigir)
- **Arquivos a Modificar:** [Lista de arquivos específicos]
- **Banco de Dados:** [Necessário rodar script de correção de dados?]
- **Riscos da Correção:** [O que pode quebrar ao consertar isso?]

## Plano de Ação (Correção Cirúrgica)
1. [Passo 1: Ajustar a query/lógica no arquivo X]
2. [Passo 2: Adicionar teste unitário para cobrir o caso]
3. [Passo 3: Validar comportamento]

## Testes Necessários
- **Validation Gates:** [O que testar para garantir que o bug sumiu]
- **Testes de Regressão:** [O que testar para garantir que não quebrou o resto]

## Próximas Etapas
1. Revisar e aprovar este Bugfix Design
2. Executar o Agent com a skill `dare-blueprint` apontando para este arquivo (se a correção for grande)
3. Ou ir direto para a skill `dare-tasks` se for simples
```

### Passo 5: Pedir Aprovação
Após gerar o Design, crie um Artifact do tipo Implementation Plan e peça ao usuário para revisar o diagnóstico e a abordagem da correção.

## Regras de Ouro para Bugfixes

1. **Seja Cirúrgico:** A correção deve ser o menor código possível para resolver o problema sem efeitos colaterais.
2. **Causa Raiz:** Foque na origem do problema, não no sintoma.
3. **Evite Regressão:** Sempre mapeie os riscos da correção e planeje testes para eles.
4. **Adicione Testes:** Se o bug ocorreu, é porque faltava um teste. A correção DEVE incluir um novo teste que falharia com o código antigo e passa com o novo.
