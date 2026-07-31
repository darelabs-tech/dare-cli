# 035 — Engine AST nativo

**Objetivo:** Criar extracao de endpoints e entidades usando tree-sitter nativo.

**Posicao na sequencia:** 35 de 56

## Resultado esperado

Ao concluir este microplano, o projeto tera uma entrega verificavel relacionada a **engine ast nativo**, sem depender de etapas futuras para demonstrar o comportamento principal definido neste escopo.

## Pre-requisitos

- [ ] Microplanos 005 e 009 concluidos

## Escopo incluido

- [ ] Adicionar gramaticas TS/TSX/JS/Python/PHP/Go/Ruby/Rust
- [ ] Criar parser por linguagem
- [ ] Extrair endpoints
- [ ] Extrair classes/models/entities
- [ ] Implementar fallback regex
- [ ] Deduplicar AST e regex
- [ ] Adicionar corpus por linguagem
- [ ] Controlar feature flags se necessario

## Fora de escopo

- Funcionalidades pertencentes a microplanos posteriores.
- Mudancas de contrato sem ADR aprovado.
- Otimizacoes prematuras sem benchmark ou requisito mensuravel.

## Crates e caminhos principais

- `crates/dare-ast`

## Plano de implementacao detalhado

### 1. Preparacao

- [ ] Criar issue principal e subtarefas rastreaveis.
- [ ] Identificar fixtures e golden outputs da versao TypeScript relacionados ao escopo.
- [ ] Confirmar dependencias entre crates e impedir ciclos arquiteturais.
- [ ] Definir os erros e exit codes antes de implementar o happy path.

### 2. Implementacao

- [ ] Implementar e revisar: Adicionar gramaticas TS/TSX/JS/Python/PHP/Go/Ruby/Rust.
- [ ] Implementar e revisar: Criar parser por linguagem.
- [ ] Implementar e revisar: Extrair endpoints.
- [ ] Implementar e revisar: Extrair classes/models/entities.
- [ ] Implementar e revisar: Implementar fallback regex.
- [ ] Implementar e revisar: Deduplicar AST e regex.
- [ ] Implementar e revisar: Adicionar corpus por linguagem.
- [ ] Implementar e revisar: Controlar feature flags se necessario.

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

- [ ] Cada linguagem possui fixture.
- [ ] Fallback funciona sem grammar.
- [ ] Output e deterministico.
- [ ] `cargo fmt --check`, `cargo clippy` e `cargo test` aprovados.
- [ ] Nenhuma diferenca de compatibilidade sem classificacao.
- [ ] Release ou artefato de CI instalavel produzido.

## Entregaveis

- Implementacao revisada de **Engine AST nativo**.
- Testes automatizados e fixtures.
- Documentacao e release notes.
- Registro de decisoes ou incompatibilidades, se houver.

## Proximo microplano

Quando todos os criterios acima estiverem concluidos, avance para [`036-reverse.md`](036-reverse.md).
