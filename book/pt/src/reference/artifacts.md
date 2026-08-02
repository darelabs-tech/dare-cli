# Artefatos DARE

O DARE CLI baseia o seu fluxo de desenvolvimento na persistência de arquivos Markdown estruturados no diretório `DARE/`. Esses arquivos mantêm o contexto do projeto, permitindo que a IA entenda o escopo sem alucinações.

---

## Estrutura do Diretório de Governança

Abaixo está o inventário de arquivos e diretórios manipulados pelo DARE CLI:

| Caminho do Artefato | Gerado por | Fase / Método | Propósito |
|---|---|---|---|
| `DARE/DESIGN.md` | `dare design` | Fase 1: Design | Requisitos funcionais (tabela), stakeholders, escopo do sistema. |
| `DARE/BLUEPRINT.md` | `dare blueprint` | Fase 2: Architect | Arquitetura física (Mermaid), trade-offs, modelo de dados, contratos. |
| `DARE/TASKS.md` | `dare blueprint` | Fase 2: Architect | Lista consolidada de tasks atômicas a implementar e seus status. |
| `DARE/dare-dag.yaml` | `dare blueprint` | Fase 2: Architect | Definição formal em YAML do Grafo Acíclico Dirigido das dependências das tasks. |
| `DARE/EXECUTION/task-*.md` | `dare blueprint` | Fase 4: Execute | Especificação individual, regras e testes de cada task do DAG. |
| `DARE/TELEMETRY.md` | `dare execute` | Fase 4: Execute | Registro de tokens gastos por modelo, tempo de execução e tentativas de conserto por task. |
| `DARE/PROJECT-DNA.md` | `dare dna` | Fase 0: Brownfield | Convenções de nomenclatura, bibliotecas e estilos extraídos do legado. |
| `DARE/REVERSE/module-*.md` | `dare reverse` | Fase 0: Brownfield | Análise de engenharia reversa por módulo legado detectado. |
| `DARE/REVERSE/reverse-facts.json`| `dare reverse` | Fase 0: Brownfield | Estrutura consolidada de fatos de módulos para importações. |
| `DARE/MIGRATION/MIGRATION.md` | `dare migrate` | Fase 0: Brownfield | Estratégia de migração e transição dividida em 3 fases estruturadas. |
| `DARE/MIGRATION/parity/*.feature` | `dare migrate` | Fase 0: Brownfield | Cenários de teste BDD (Gherkin) para garantir paridade legado-alvo. |

---

## Políticas e Segurança dos Artefatos

1. **Assinatura e Integridade (`dare guard`):**
   Se o bloco `signing.enabled` estiver ativado no `dare.config.json`, qualquer modificação em um arquivo da pasta `DARE/` exige a atualização do arquivo correspondente com extensão `.minisig` (ex: `DARE/BLUEPRINT.md.minisig`) gerado com a chave Ed25519 correta. Caso contrário, o `dare guard` rejeita a execução (bloqueia o pipeline com exit code 6).
2. **Preservação de Conteúdo (`dare:managed`):**
   Para evitar que a IA sobrescreva alterações e ajustes finos que você fez manualmente nos documentos gerados, basta remover o marcador `<!-- dare:managed -->` da primeira linha útil do arquivo Markdown. A CLI respeitará o seu ajuste e irá pular esse arquivo nos builds futuros, a menos que `--force` seja passado.
