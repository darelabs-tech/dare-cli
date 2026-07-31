# 034 — Guard

**Objetivo:** Implementar Unicode, prompt injection e proveniencia com assinatura.

**Posicao na sequencia:** 34 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **guard**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 005 e 006 concluidos

## Escopo incluido

- [ ] Detectar zero-width, bidi, variation selectors, tags e homoglyphs
- [ ] Implementar modos strip e block
- [ ] Carregar scan-rules.json
- [ ] Aplicar regras de injection
- [ ] Classificar proveniencia
- [ ] Validar trustedPaths
- [ ] Assinar/verificar minisign ou Ed25519
- [ ] Retornar exit code 6
- [ ] Integrar preflight do execute agent

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-guard`
- `assets/rules/scan-rules.json`
- `crates/dare-cli/src/commands/guard.rs`

## Comandos ou superficies afetadas

- `dare guard`
- `dare guard --staged`
- `dare guard --all`
- `dare guard --sign`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Detectar zero-width, bidi, variation selectors, tags e homoglyphs.
- [ ] Implementar e revisar: Implementar modos strip e block.
- [ ] Implementar e revisar: Carregar scan-rules.json.
- [ ] Implementar e revisar: Aplicar regras de injection.
- [ ] Implementar e revisar: Classificar proveniencia.
- [ ] Implementar e revisar: Validar trustedPaths.
- [ ] Implementar e revisar: Assinar/verificar minisign ou Ed25519.
- [ ] Implementar e revisar: Retornar exit code 6.
- [ ] Implementar e revisar: Integrar preflight do execute agent.

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

- [ ] Corpus malicioso e detectado.
- [ ] Evidencias sao redigidas.
- [ ] Assinatura invalida falha.
- [ ] Agent nao inicia apos FAIL.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Guard**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`035-engine-ast-nativo.md`](035-engine-ast-nativo.md).
