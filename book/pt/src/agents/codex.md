# Codex (OpenAI)

Integração oficial entre a metodologia DARE e o **Codex** (OpenAI) através de adaptadores nativos na crate `dare-harness`.

## Uso

```bash
dare harness codex [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--force` | Força a reinstalação e o overwrite de arquivos existentes mesmo que tenham sido editados ou customizados manualmente pelo usuário (sem a marcação `managed`) |

---

## O que a instalação faz?

Ao rodar o comando `dare harness codex`, a CLI executa o seguinte pipeline de materialização de recursos:

1. **Geração do `AGENTS.md`:**
   - Cria o arquivo `AGENTS.md` na raiz do projeto contendo um catálogo dinâmico das habilidades (skills) ativas.
   - O arquivo possui links rápidos e descrições das skills para facilitar a invocação direta pelo desenvolvedor ou pelo agente no padrão `$skill-name`.
2. **Instalação das Habilidades de IA (Skills):**
   - Carrega as capacidades da matriz de capacidades do DARE (`capability-matrix.yml`) contendo **49 capacidades**.
   - Materializa os arquivos de skills sob `.codex/skills/{id}/SKILL.md`.
   - Por consistência e para evitar divergência de conteúdo no projeto, a CLI realiza a escrita dupla (**shared skills**), copiando as mesmas especificações sob a pasta comum de agentes `.agents/skills/{id}/SKILL.md` (compartilhada com o harness do Antigravity).

---

## Política de Preservação e Sobrescrita

O DARE CLI respeita as customizações locais do desenvolvedor:
- **`managed` (Gerenciado):** Arquivos contendo os marcadores `<!-- dare:managed -->` ou que iniciam com o bloco de frontmatter `---` são identificados como gerenciados e serão atualizados automaticamente a cada execução do harness.
- **`customized` (Modificado):** Se o usuário remover o marcador ou alterar a primeira linha do arquivo, a CLI assume que o arquivo foi customizado e irá **pular** a sua atualização nas próximas rodadas (evitando apagar customizações manuais do desenvolvedor). Para forçar o overwrite destes arquivos, deve-se passar a flag `--force`.

---

## Verificação e Diagnóstico

```bash
# Executa a verificação de integridade dos artefatos do Codex
dare info
```

- Varre os arquivos instalados (como a existência de `AGENTS.md` e das pastas de skills correspondentes) comparando com a matriz de capacidades do DARE (`capability-matrix.yml`).
- Caso detecte a ausência de algum arquivo necessário, exibe a lista contendo as primeiras pendências localizadas e recomenda a reinstalação.
