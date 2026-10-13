# APPROVAL

- 2026-10-12: o usuário revisou DESIGN, BLUEPRINT e TASKS e aprovou a execução ("revisado e aprovado pode seguir").
- Diretriz anterior do usuário: replicar o npm 3.18.1, mas corrigir os bugs (B-01..B-09), com as diferenças documentadas.
- Docker (confirmado pelo usuário): o init fica igual ao npm. Sem `Dockerfile`/`docker-compose.yml` gravados no init; o Docker é usado como toolchain dos scaffolders (auto/native/docker) e na fase 1 de containerização via `/dare-docker`.
- Ponto sem resposta explícita; seguido o padrão recomendado:
  - `--fullstack react|vue` (forma do Rust 4.0) é rejeitado com uma mensagem que orienta usar o modo interativo (monorepo).
- A aprovação não cobre merge na `main`.
