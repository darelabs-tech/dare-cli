# Claude Code

Integração oficial entre a metodologia DARE e o **Claude Code** (agente de IA baseado em CLI da Anthropic) através de adaptadores nativos na crate `dare-harness`.

## Uso

```bash
dare harness claude [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--force` | Força a reinstalação e o overwrite de arquivos existentes mesmo que tenham sido editados ou customizados manualmente pelo usuário (sem a marcação `managed`) |

---

## O que a instalação faz?

Ao rodar o comando `dare harness claude`, a CLI executa o seguinte pipeline de materialização de recursos:

1. **Geração do `CLAUDE.md`:**
   - Cria o arquivo `CLAUDE.md` na raiz do projeto contendo as instruções iniciais e apontamentos para a pasta de comandos locais do Claude (`.claude/commands/`).
   - Escreve na primeira linha o marcador `<!-- dare:managed -->` para rastreamento de atualizações.
2. **Instalação de Comandos de Capacidade:**
   - Carrega as capacidades da matriz de capacidades do DARE (`capability-matrix.yml`) contendo **49 comandos** suportados.
   - Gera um script executável correspondente para cada comando do DARE CLI sob o diretório `.claude/commands/` (ex.: `.claude/commands/dare-design`).
3. **Configuração de Configurações (`settings.json`):**
   - Cria ou atualiza o arquivo `.claude/settings.json`.
   - Adiciona a flag de controle de gerenciamento do DARE: `"_dare_managed": true`.
   - Injeta a regra `postToolUse` contendo um comando fixo de eco e aviso do Ralph Loop (`echo "..."`). Esse comando é acionado após a IA utilizar ferramentas e serve para steering e lembrete do Ralph Loop para o agente.

---

## Política de Preservação e Sobrescrita

O DARE CLI respeita as customizações locais do desenvolvedor:
- **`managed` (Gerenciado):** Arquivos contendo os marcadores `<!-- dare:managed -->` ou a propriedade `"_dare_managed": true` no JSON são identificados como gerenciados e serão atualizados automaticamente a cada execução do harness.
- **`customized` (Modificado):** Se o usuário remover a propriedade ou marcador de um arquivo, a CLI assume que o arquivo foi customizado e irá **pular** a sua atualização nas próximas rodadas. Para forçar o overwrite, deve-se utilizar a flag `--force`.

---

## Verificação e Diagnóstico

O DARE CLI valida o estado atual de instalação do harness:

```bash
# Executa a verificação de integridade dos artefatos do Claude Code
dare info
```

- Varre os arquivos instalados comparando com a matriz de capacidades do DARE (`capability-matrix.yml`).
- Caso detecte a ausência de algum arquivo necessário, exibe a lista contendo as primeiras pendências localizadas e recomenda a reinstalação.
