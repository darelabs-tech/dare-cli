# Visão Geral dos Comandos

O DARE CLI oferece comandos organizados em grupos funcionais.

---

## Inicialização e Setup

| Comando | Descrição |
|---|---|
| [`dare init`](init.md) | Inicializa a infraestrutura DARE num projeto novo (greenfield) |
| [`dare bootstrap`](bootstrap.md) | Materializa o scaffold da stack no projeto atual |
| [`dare discover`](discover.md) | Detecção brownfield — instala o DARE em projetos existentes |
| [`dare welcome`](welcome.md) | Exibe o banner de boas-vindas e guia de início rápido |

## O Método DARE

| Comando | Descrição |
|---|---|
| [`dare design`](design.md) | Gera `DARE/DESIGN.md` a partir de requisitos |
| [`dare blueprint`](blueprint.md) | Gera `DARE/BLUEPRINT.md` com arquitetura e tasks |
| [`dare execute`](execute.md) | Executa uma task com Ralph Loop |
| [`dare review`](review.md) | Audita implementação contra as specs |

## DAG e Execução

| Comando | Descrição |
|---|---|
| `dare dag next` | Próxima task disponível no grafo |
| `dare dag status` | Status de todas as tasks |
| `dare dag visualize` | Visualiza o grafo no terminal |
| `dare dag complete` | Marca task como concluída |
| `dare dag fail` | Marca task como falha |
| `dare dag reset` | Reseta task para ready |
| `dare dag watch` | Modo watch (atualiza a cada 2s) |
| [`dare validate`](validate.md) | Valida `dare-dag.yaml` (ciclos, refs quebradas) |

## Engenharia Reversa (Brownfield)

| Comando | Descrição |
|---|---|
| [`dare reverse`](reverse.md) | Engenharia reversa de projetos legados |
| [`dare dna`](dna.md) | Extrai convenções do projeto para `PROJECT-DNA.md` |
| [`dare migrate`](migrate.md) | Estratégia de migração de legado |

## Qualidade e Segurança

| Comando | Descrição |
|---|---|
| [`dare refine`](refine.md) | Quebra tasks complexas em sub-tasks menores |
| [`dare guard`](guard.md) | Gate de segurança OWASP sobre artefatos |
| [`dare bench`](bench.md) | Harness de benchmarks e fix-rate |

## Skills e Agentes

| Comando | Descrição |
|---|---|
| [`dare skill`](skill.md) | Gerencia skills (adicionar, remover, publicar) |

## IA e Enriquecimento

| Comando | Descrição |
|---|---|
| [`dare ai`](ai.md) | Enriquecimento semântico via LLM providers |

## Infraestrutura

| Comando | Descrição |
|---|---|
| [`dare update`](update.md) | Sincroniza templates e skills com a versão instalada |
| [`dare self`](self.md) | Gerencia a instalação do próprio CLI (`dare self update`) |
| [`dare info`](info.md) | Status de integridade e versão |
| [`dare graph`](graph.md) | Consulta o grafo de conhecimento do projeto |
| [`dare dashboard`](dashboard.md) | Painel de telemetria local (browser) |

---

## Flags globais

Disponíveis em todos os comandos:

| Flag | Descrição |
|---|---|
| `--json` | Saída em JSON estruturado (para scripts/agentes) |
| `--no-color` | Desativa cores ANSI |
| `--no-banner` | Suprime o banner de boas-vindas |
| `-v, --verbose` | Aumenta verbosidade dos logs |
| `--dir <path>` | Define o diretório raiz do projeto |
| `-h, --help` | Exibe ajuda do comando |
| `-V, --version` | Exibe versão do CLI |

---

## Exit Codes

Todos os comandos seguem a mesma convenção de exit codes. Veja [Exit Codes](../reference/exit-codes.md).
