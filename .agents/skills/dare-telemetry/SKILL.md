---
name: dare-telemetry
description: Rastreamento de tokens e modelos de IA consumidos em cada etapa do Método DARE. Gera DARE/TELEMETRY.md com modelo usado, tokens estimados, tempo e tentativas (Ralph Loop) por comando — para auditoria, monitoramento de uso e otimização de custos.
---

# DARE Telemetry Skill

Você é um especialista em observabilidade e monitoramento de uso de IA. Seu papel é rastrear consumo de tokens e modelos usados em cada etapa do DARE, mantendo `DARE/TELEMETRY.md` atualizado.

> **Diferença com `dare-quality-telemetry`:**
> - `dare-telemetry` (esta) — rastreia **uso de IA** (modelos, tokens, tempo) por comando DARE
> - `dare-quality-telemetry` — agrega **métricas de qualidade** das skills (M-01 a M-04)

## Quando usar

- Ao final de cada comando DARE executado
- Para gerar relatório periódico de uso de IA
- Para auditoria de compliance (qual IA foi usada, onde)
- Para otimização de custos (qual etapa consome mais)

## Modelos rastreáveis

Independente do IDE/agente:

| Modelo | Provider | Características | Melhor para |
|---|---|---|---|
| Claude Opus 4.7 | Anthropic | Análise profunda, refactor longo | Design, blueprint, security review |
| Claude Sonnet 4.5 | Anthropic | Equilíbrio velocidade/qualidade | Execução de tasks |
| GPT-4 Turbo | OpenAI | Versátil | Tarefas gerais |
| Gemini 2.0 Flash | Google | Ultra rápido | Tasks simples, processamento em batch |
| Modelos locais (Ollama) | Self-hosted | Privacidade total | Dados sensíveis |

## Estrutura `DARE/TELEMETRY.md`

```markdown
# Telemetria do Projeto: [Nome]

## Resumo Executivo
- **Projeto:** [Nome]
- **Data de início:** [ISO 8601]
- **Tokens totais processados:** [Número]
- **Modelos utilizados:** [Lista]
- **Tempo total de execução:** [Tempo]
- **Custo estimado:** $[X]

## Detalhamento por Etapa

### 1. Design (`/dare-design`)
- **Data/Hora:** [Timestamp]
- **Modelo:** Claude Opus 4.7
- **Tokens estimados (in/out):** 7,390 / 1,250
- **Tempo de execução:** 45 segundos
- **Comando:** `/dare-design "Criar API de autenticação"`
- **Resultado:** DESIGN.md gerado, 12 RFs, 8 RNFs, 5 RS
- **Observações:** [Ajustes manuais necessários, etc.]

### 2. Blueprint (`/dare-blueprint`)
- **Data/Hora:** [Timestamp]
- **Modelo:** Claude Opus 4.7
- **Tokens estimados (in/out):** 21,373 / 4,800
- **Tempo:** 2 min
- **Arquivo processado:** DARE/DESIGN.md
- **Resultado:** BLUEPRINT.md com 8 fases, 25 tabelas, 4 diagramas Mermaid

### 3. Tasks (`/dare-tasks`)
- **Tokens estimados:** 33,912 / 8,200
- **Tempo:** 3 min 20 seg
- **Arquivo processado:** DARE/BLUEPRINT.md
- **Tasks geradas:** 12

### 4. Execute Tasks (`/dare-execute`)

- **task-001 — Migration de Users**
  - Modelo: Claude Sonnet 4.5
  - Tokens: 7,801 / 2,500
  - Tempo: 1 min 30 seg
  - Tentativas (Ralph Loop): 1
  - Status: ✓ Sucesso

- **task-002 — AuthController**
  - Modelo: Claude Sonnet 4.5
  - Tokens: 11,357 / 3,200
  - Tempo: 2 min
  - Tentativas: 2 (1 falha por typo)
  - Status: ✓ Sucesso

## Análise

| Etapa | Tokens in/out | % do total | Tempo |
|---|---|---|---|
| Design | 7,390 / 1,250 | 5% | 45 seg |
| Blueprint | 21,373 / 4,800 | 15% | 2 min |
| Tasks | 33,912 / 8,200 | 24% | 3 min 20 seg |
| Execute (12 tasks) | 85,234 / 24,000 | 56% | 25 min |
| **TOTAL** | **147,909 / 38,250** | **100%** | **~31 min** |

## Modelos utilizados

- Claude Opus 4.7: 62,675 tokens (42%) — design + blueprint + tasks
- Claude Sonnet 4.5: 85,234 tokens (58%) — execução de tasks

## Custo estimado

| Modelo | Tokens in | Tokens out | $/M in | $/M out | Total |
|---|---|---|---|---|---|
| Opus 4.7 | 62,675 | 14,250 | $15 | $75 | $2.01 |
| Sonnet 4.5 | 85,234 | 24,000 | $3 | $15 | $0.62 |
| **Total** | | | | | **$2.63** |
```

## Como aplicar

### Passo 1: Após `/dare-design`

Adicione entrada na seção Design com:
- Timestamp
- Modelo do agente (perguntar ao usuário se não souber automaticamente)
- Tokens estimados (in/out) — pegar do output do modelo se disponível
- Tempo de execução
- Comando executado
- Observações qualitativas

### Passo 2: Após `/dare-blueprint`

Idem. Adicione tabelas e diagramas gerados como resultado.

### Passo 3: Após `/dare-tasks`

Inclua número de tasks geradas, complexidade média.

### Passo 4: Após cada `/dare-execute`

Para cada task:
- Modelo usado
- Tokens
- Tempo
- **Tentativas no Ralph Loop** (importante — se >2, task precisa refino)
- Status (Sucesso/Falha)

### Passo 5: Recompilar análise + custos

A cada 5 tasks ou release, atualize a seção Análise com totais agregados.

## Capturando informações do agente

Cada IDE expõe diferente:

**Antigravity:**
- Modelo aparece no header da conversa
- Tokens estimados em `?ax-debug` ou settings

**Claude Code:**
- `/cost` mostra tokens da sessão
- Modelo no rodapé

**Cursor:**
- Barra de status canto inferior direito
- Histórico de conversa tem detalhes por resposta

## Otimizações recomendadas

1. **Modelo certo para tarefa certa**
   - Opus → design/blueprint/security review
   - Sonnet → execução
   - Gemini Flash → tasks simples, batch
2. **Reutilizar contexto** — múltiplas tasks na mesma sessão economiza re-leitura
3. **Revisar Blueprint antes de Tasks** — evita re-geração
4. **Agrupar tasks relacionadas** — contexto reaproveitado
5. **Ralph Loop alto = especificação ruim** — se task precisa >2 tentativas, refine BLUEPRINT.md

## Antipatterns

| AP | Antipattern | Por quê |
|---|---|---|
| AP-01 | Não rastrear telemetria | Sem dados para otimizar |
| AP-02 | Usar Opus para tasks triviais | Desperdício de custo |
| AP-03 | Ignorar Ralph Loop alto | Specs ruins se acumulam |
| AP-04 | Não auditar modelo (compliance) | Risco regulatório se houver |

## Dicas

- **Combine** com `dare-quality-telemetry` — esta rastreia uso de IA, aquela rastreia qualidade
- **Exporte** TELEMETRY.md para dashboard (Grafana, Notion, etc.) periodicamente
- **Trate custo como métrica** — adicione ao Ralph Loop como gate (>$Y/feature = revisar specs)

---

Esta skill é parte do DARE Method e está sob licença MIT.
