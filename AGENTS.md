# AGENTS.md

> Arquivo gerado por `/init` com análise automática. Edite manualmente para ajustar convenções.

## Projeto

- **Nome:** monitor-tray
- **Descrição:** Aplicativo de monitoramento de sistema para Linux que exibe métricas em tempo real na system tray, com foco em performance, baixo consumo de recursos e menu detalhado com CPU, memória, disco, rede e uptime.

## Stack

- **Linguagem(s):** Rust (Cargo.toml com edition 2021; README indica Rust 1.70+)
- **Frameworks:** GTK, libappindicator, Tokio

## Gerenciamento de Dependências

- **Instalar tudo:** `(preencher manualmente)`
- **Adicionar pacote:** `(preencher manualmente)`
- **Remover pacote:** `(preencher manualmente)`

## Comandos Essenciais

- **Dev server:** `make dev`
- **Build:** `cargo build --release`
- **Testes:** `make test` ou `cargo test`

## Estrutura de Diretórios

- **Código principal:** `src/`
- **Testes:** `src/` com módulos `#[cfg(test)]`; `tests/` reservado para integração futura

## Módulos

- **`src/main.rs`** — Inicializa a aplicação GTK/libappindicator, cria o menu da tray, gera ícones SVG dinâmicos e coordena a atualização periódica da interface.
- **`src/monitor.rs`** — Encapsula a coleta de métricas do sistema via `sysinfo` e define os modelos serializáveis de CPU, memória, disco, rede e sistema.

## Arquitetura

- **Estilo:** Flat modular
- **Descrição:** `main.rs` concentra a camada de apresentação e orquestração da aplicação desktop de tray. `monitor.rs` funciona como camada de coleta e modelagem de dados, retornando `SystemMetrics` consumido pela interface.

## Testes

- **Framework:** `cargo test` (testes unitários Rust)
- **Diretório:** `src/` com módulos `#[cfg(test)]` embutidos; `tests/` reservado para integração futura
- **Executar todos:** `make test` ou `cargo test`
- **Com cobertura:** `(preencher manualmente)`

## Convenções de Código

- **Tamanho máximo de função:** 150 linhas
- **Tamanho máximo de arquivo:** 700 linhas
- **Aninhamento máximo:** 3 níveis
- **Docstrings / comentários:** Português brasileiro
- **Identificadores (variáveis, funções, classes):** Inglês
- Rust: módulos separados por arquivo, `struct`s tipadas para dados de domínio e `Result` para erros recuperáveis

## Commits

Este projeto segue o padrão **Conventional Commits**.
Antes de commitar, carregue a skill de commit:

```
/skill:git-commit-push
```

Ou siga diretamente as regras em `.agents/skills/git-commit-push/SKILL.md`.

## Agentes e Skills

| Agente    | Função                                         | Modo                   |
|-----------|------------------------------------------------|------------------------|
| `build`   | Implementa funcionalidades e corrige bugs      | escrita completa       |
| `ask`     | Responde perguntas somente-leitura             | somente-leitura        |
| `plan`    | Cria planos detalhados em `.pi/plans/`         | escrita em .pi/plans/  |
| `quality` | Auditoria de qualidade de código               | bash + leitura         |
| `qa`      | Análise de bugs e edge cases                   | bash + leitura         |
| `test`    | Cria e mantém testes automatizados             | escrita em `src/` e `tests/` |
| `doc`     | Cria documentação técnica em `docs/`           | escrita em docs/       |
