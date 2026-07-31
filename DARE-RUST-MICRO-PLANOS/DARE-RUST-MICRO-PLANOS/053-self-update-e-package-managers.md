# 053 — Self-update e package managers

**Objetivo:** Completar distribuicao nativa, atualizacao, rollback e desinstalacao.

**Posicao na sequencia:** 53 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **self-update e package managers**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplano 015 concluido
- [ ] Release beta estabilizada

## Escopo incluido

- [ ] Implementar self update por canal/versao
- [ ] Adicionar lock e download temporario
- [ ] Verificar checksum e assinatura
- [ ] Trocar binario atomicamente
- [ ] Implementar rollback e uninstall
- [ ] Criar Homebrew tap
- [ ] Criar WinGet ou Scoop
- [ ] Testar upgrades entre releases

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-cli/src/commands/self_update.rs`
- `packaging/homebrew`
- `packaging/winget`
- `packaging/scoop`

## Comandos ou superficies afetadas

- `dare self update`
- `dare self rollback`
- `dare self uninstall`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Implementar self update por canal/versao.
- [ ] Implementar e revisar: Adicionar lock e download temporario.
- [ ] Implementar e revisar: Verificar checksum e assinatura.
- [ ] Implementar e revisar: Trocar binario atomicamente.
- [ ] Implementar e revisar: Implementar rollback e uninstall.
- [ ] Implementar e revisar: Criar Homebrew tap.
- [ ] Implementar e revisar: Criar WinGet ou Scoop.
- [ ] Implementar e revisar: Testar upgrades entre releases.

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

- [ ] Upgrade interrompido preserva versao anterior.
- [ ] Assinatura invalida bloqueia instalacao.
- [ ] Package managers instalam versao correta.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Self-update e package managers**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`054-hardening-de-paridade-e-seguranca.md`](054-hardening-de-paridade-e-seguranca.md).
