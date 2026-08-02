# Cursor

Integração oficial entre a metodologia DARE e o editor **Cursor** através de adaptadores nativos na crate `dare-harness`.

## Uso

```bash
dare harness cursor [OPTIONS]
```

## Flags

| Flag | Descrição |
|---|---|
| `--force` | Força a reinstalação e o overwrite de arquivos existentes mesmo que tenham sido editados ou customizados manualmente pelo usuário (sem a marcação `managed`) |

---

## O que a instalação faz?

Ao rodar o comando `dare harness cursor`, a CLI executa o seguinte pipeline de materialização de recursos:

1. **Geração do `.cursorrules`:**
   - Cria o arquivo `.cursorrules` na raiz do projeto contendo as regras de comportamento do assistente.
   - Escreve na primeira linha o marcador `<!-- dare:managed -->` para rastreamento de atualizações.
2. **Instalação de Comandos de Capacidade:**
   - Carrega as capacidades da matriz de capacidades do DARE (`capability-matrix.yml`) contendo **49 comandos** suportados.
   - Gera um script executável correspondente para cada comando sob o diretório `.cursor/commands/` (ex.: `.cursor/commands/dare-design`). Os scripts utilizam o renderizador partilhado do Claude (`render_claude_command`), garantindo que o corpo do markdown dos comandos seja idêntico.

---

## Exceções de Paridade e Limitações (Classe C)

Devido a restrições e mudanças de design do ecossistema do Cursor:
- **Rules `.mdc`:** A geração de regras individuais estruturadas em arquivos `.mdc` foi **deferida** nesta versão do DARE CLI, pois a API e o inventário de assets ainda não estão consolidados na stack estável de testes.
- **Diferença de Comandos:** O baseline legado da CLI Cursor rascunhava 33 comandos e 25 regras, mas a implementação em Rust congela a fonte única de verdade (SoT) a partir do `capability-matrix.yml` que contém **49 comandos** completos, garantindo paridade com os outros adaptadores.

---

## Política de Preservação e Sobrescrita

O DARE CLI respeita as customizações locais do desenvolvedor:
- **`managed` (Gerenciado):** Arquivos contendo os marcadores `<!-- dare:managed -->` são atualizados automaticamente.
- **`customized` (Modificado):** Se o usuário remover o marcador de um arquivo, a CLI assume que o arquivo foi customizado e irá **pular** a sua atualização nas próximas rodadas. Para forçar o overwrite, utilize `--force`.

---

## Verificação e Diagnóstico

```bash
# Executa a verificação de integridade dos artefatos do Cursor
dare info
```

- Varre os arquivos instalados comparando com a matriz de capacidades do DARE (`capability-matrix.yml`).
- Caso detecte a ausência de algum comando necessário, exibe a lista contendo as primeiras pendências localizadas e recomenda a reinstalação.
