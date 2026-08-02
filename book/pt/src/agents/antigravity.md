# Antigravity

Integração oficial entre a metodologia DARE e o **Antigravity** (extensão Gemini para IDEs de desenvolvimento) através de adaptadores nativos na crate `dare-harness`.

## Uso

```bash
dare harness antigravity [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--force` | Força a reinstalação e o overwrite de arquivos existentes mesmo que tenham sido editados ou customizados manualmente pelo usuário (sem a marcação `managed`) |

---

## O que a instalação faz?

Ao rodar o comando `dare harness antigravity`, a CLI executa o seguinte pipeline de materialização de recursos:

1. **Configuração de Regras (`antigravityrules`):**
   - Cria o arquivo de regras locais de comportamento do assistente.
   - Escreve na primeira linha o marcador `<!-- dare:managed -->` para rastreamento de atualizações.
2. **Diretório de Workflows:**
   - Cria a pasta `.agents/workflows/` contendo um arquivo `.gitkeep` vazio, preparando a infraestrutura para automações do agente.
3. **Instalação das Habilidades de IA (Skills):**
   - Materializa as especificações estruturadas de habilidades que a IA consome no diretório local `.agents/skills/{id}/SKILL.md` (compatível e compartilhado com o harness do Codex).
   - Valida se o frontmatter de cada `SKILL.md` gerado contém chaves `name` e `description` válidas e preenchidas para indexação correta na IDE.

---

## Política de Preservação e Sobrescrita

O DARE CLI respeita as customizações locais do desenvolvedor:
- **`managed` (Gerenciado):** Arquivos contendo os marcadores `<!-- dare:managed -->` ou a estrutura inicial `---` do frontmatter do DARE são identificados como gerenciados e serão atualizados automaticamente a cada execução do harness.
- **`customized` (Modificado):** Se o usuário apagar o marcador managed de um arquivo de configuração ou skill, a CLI assume que o arquivo é customizado e irá **pular** a sua atualização nas próximas rodadas (evitando apagar customizações manuais do desenvolvedor). Para forçar o overwrite destes arquivos, deve-se passar a flag `--force`.

---

## Verificação e Diagnóstico

O DARE CLI pode validar o estado atual de instalação do harness:

```bash
# Executa a verificação de integridade dos artefatos do Antigravity
dare info
```

- Varre os arquivos instalados comparando com a matriz de capacidades do DARE (`capability-matrix.yml` que contém a lista consolidada de capacidades e comandos expostos).
- Caso detecte a ausência de algum arquivo necessário, exibe a lista contendo as primeiras pendências localizadas e recomenda a reinstalação.
