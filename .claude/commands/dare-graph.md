# /dare-graph

Consulta e visualiza o grafo de conhecimento do projeto (tasks, arquivos, schemas, endpoints, componentes, entidades e suas relações).

> Este comando expõe o CLI `dare graph` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- Você quer entender dependências entre tasks/arquivos/entidades.
- Precisa achar nós relacionados a um termo, ou exportar um diagrama do grafo.

## Como rodar

```bash
dare graph stats                       # contagem de nós/arestas por tipo
dare graph query <termo> --limit 10    # busca nós por label/descrição
dare graph query auth --type endpoint
dare graph viz --format mermaid -o graph.mmd
dare graph ingest                      # re-sincroniza o grafo do dare-dag.yaml
```

## O que fazer

1. Escolha o subcomando conforme a intenção: `stats`, `query <termo>`, `viz`, `ingest`.
2. Rode o comando e interprete a saída (para `viz`, abra/renderize o diagrama gerado).
3. Se o grafo parecer desatualizado, rode `dare graph ingest` para re-sincronizar.

## Comandos relacionados

`/dare-dag` · `/dare-execute`
