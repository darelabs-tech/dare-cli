# 015 — Pipeline de release nativo alpha

**Objetivo:** Publicar binarios instalaveis sem npm desde o primeiro ciclo funcional.

**Posicao na sequencia:** 15 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **pipeline de release nativo alpha**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 003 concluido

## Escopo incluido

- [ ] Criar workflow de tags alpha
- [ ] Empacotar tar.gz e zip por target
- [ ] Gerar SHA256SUMS
- [ ] Gerar SBOM SPDX
- [ ] Assinar checksums
- [ ] Criar install.sh e install.ps1 iniciais
- [ ] Executar smoke test de instalacao limpa

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `.github/workflows/release.yml`
- `installers/install.sh`
- `installers/install.ps1`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Criar workflow de tags alpha.
- [ ] Implementar e revisar: Empacotar tar.gz e zip por target.
- [ ] Implementar e revisar: Gerar SHA256SUMS.
- [ ] Implementar e revisar: Gerar SBOM SPDX.
- [ ] Implementar e revisar: Assinar checksums.
- [ ] Implementar e revisar: Criar install.sh e install.ps1 iniciais.
- [ ] Implementar e revisar: Executar smoke test de instalacao limpa.

### 3. Compatibilidade

- [ ] Comparar comportamento observavel com a versao TypeScript 3.18.1.
- [ ] Registrar diferencas intencionais no changelog e ADR correspondente.
- [ ] Preservar paths, IDs, formatos e ordenacoes deterministicas aplicaveis.
- [ ] Confirmar funcionamento em Linux, macOS e Windows quando houver I/O, processos ou paths.

### 4. Seguranca

- [ ] Validar entradas e limites.
- [ ] Aplicar path safety em toda leitura e escrita.
- [ ] Evitar shell concatenado; usar argv separado para processos.
- [ ] Redigir secrets, tokens e dados sensiveis de logs e erros.
- [ ] Testar falhas parciais, cancelamento e rollback quando aplicavel.

### 5. Documentacao e release

- [ ] Atualizar help, documentacao tecnica e exemplos.
- [ ] Atualizar matriz de compatibilidade/capabilities quando aplicavel.
- [ ] Adicionar release notes.
- [ ] Gerar binarios, checksums e smoke tests no canal atual.

## Estrategia de testes

- [ ] Testes unitarios do dominio.
- [ ] Testes de integracao com filesystem/processos reais quando aplicavel.
- [ ] Golden tests contra a implementacao TypeScript.
- [ ] Casos de erro e entradas malformadas.
- [ ] Testes cross-platform para comportamento dependente de sistema.
- [ ] Testes de seguranca relevantes ao escopo.

## Criterios de aceite

- [ ] Release alpha contem cinco targets.
- [ ] Checksum e SBOM publicados.
- [ ] Instaladores executam dare --version.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Pipeline de release nativo alpha**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`016-comando-welcome.md`](016-comando-welcome.md).
