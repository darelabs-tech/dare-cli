# Comando: /dare-init

Cria um projeto DARE do zero (greenfield) com setup interativo: escolhe stack backend/frontend, knowledge graph, IDE e gera o scaffolder completo + DNA DARE.

> Este comando expõe o CLI `dare init` na IDE. O agente pode **rodar o comando no terminal** e interpretar a saída.

## Quando usar

- O usuário quer começar um projeto **novo** do zero.
- Não existe código ainda — é greenfield. Para projeto legado, use `/dare-discover`.

## Como rodar

```bash
dare init                       # fluxo interativo completo
dare init minha-api --stack node-nestjs
dare init meu-mcp --mcp node-ts --transport http
dare init api --stack go-gin --toolchain docker --non-interactive
```

## O que fazer

1. Rode `dare init` (ou com `--stack`/`--mcp` se o usuário já decidiu a stack).
2. Responda aos prompts: nome, stack backend, frontend opcional, knowledge graph (json/sqlite/neo4j), IDE(s).
3. Ao final, o projeto tem scaffolder + os 7 artefatos de DNA + comandos/skills de IDE instalados.
4. Próximo passo: descreva a ideia com `/dare-design`.

## Comandos relacionados

`/dare-design` · `/dare-discover` · `/dare-bootstrap`
