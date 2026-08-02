# `dare skill`

Gerencia o ciclo de vida de **Skills** do DARE no projeto — permitindo listar, inspecionar, instalar, atualizar, remover ou publicar pacotes de habilidades de IA de forma segura.

## Uso

```bash
dare skill <ACTION> [OPTIONS]
```

## Ações Disponíveis

| Comando | Descrição |
|---|---|
| `list` | Lista todas as skills instaladas no projeto atual |
| `info <SKILL_NAME>` | Exibe informações detalhadas sobre uma skill específica |
| `add <SKILL_NAME>` | Instala uma skill do registro central (ou de um arquivo local) |
| `remove <SKILL_NAME>` | Desinstala uma skill do projeto atual |
| `update <SKILL_NAME>` | Atualiza uma skill instalada para a versão mais recente |
| `publish <SKILL_NAME>` | Compacta, assina e publica uma skill para o registro |

---

## O que são Skills?

Skills são pacotes estruturados que estendem a capacidade de raciocínio de agentes de IA para tarefas altamente especializadas do projeto. Elas são gravadas em `packages/skills/<SKILL_NAME>/` e contêm:
- **`skill.yml` / `SKILL.md`:** Manifesto, descrição e instruções semânticas.
- **Scripts de Suporte:** Ferramentas adicionais de automação.

---

## Instalação e Ciclo de Vida

### Fluxo de Adição (`dare skill add`)
1. Valida se o nome da skill é seguro e está bem-formado.
2. Resolve a skill e suas dependências no `CompositeRegistry`.
3. Executa download e materializa a estrutura em pasta temporária de staging (`packages/skills/.staging-<name>/`).
4. Garante que os arquivos extraídos não escapem do diretório sandbox (path safety de arquivos zip/tar).
5. Move atomicamente para `packages/skills/<SKILL_NAME>/` e registra o pacote em `.dare/skills.yml`.

### Fluxo de Remoção (`dare skill remove`)
1. Verifica se a skill está instalada.
2. Varre os manifestos de outras skills instaladas para detectar dependências inversas (reverse-dependencies).
3. Se outra skill instalada depender da skill alvo, a remoção é bloqueada com erro `InvalidInput (4)` para evitar quebras.
4. Caso não haja pendências, remove o diretório e atualiza o manifesto.

---

## Publicação Segura (`dare skill publish`)

Permite empacotar e disponibilizar uma skill criada localmente:
- **Licenciamento:** Requer licença declarada como `MIT` no manifesto da skill.
- **Compactação:** Cria um arquivo compactado `.tar.gz` contendo a estrutura da skill.
- **Assinatura Digital:** Assina o pacote gerado gerando um arquivo de assinatura `.minisig` com criptografia **Ed25519** de alta segurança. Requer a chave privada configurada na variável de ambiente `DARE_SKILL_PRIVATE_KEY`.

---

## Exemplos de Uso

```bash
# Lista todas as skills ativas no projeto
dare skill list

# Instala a skill dare-laravel-api do registro
dare skill add dare-laravel-api

# Remove uma skill instalada
dare skill remove dare-laravel-api

# Compacta e assina uma skill para publicação
dare skill publish minha-custom-skill
```

## Exit codes

| Código | Descrição |
|---|---|
| `0` | Operação concluída com sucesso |
| `2` | Uso de argumentos inválidos |
| `3` | A skill especificada não foi encontrada no registro ou no projeto |
| `4` | Entrada inválida (como licença inválida, erro de assinatura ou quebra de dependências) |
| `5` | Falha inesperada de I/O na leitura ou gravação de arquivos |
