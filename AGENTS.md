# AGENTS.md

> Arquivo gerado por `/init` com análise automática. Edite manualmente para ajustar convenções.

## Projeto

- **Nome:** monitor-tray
- **Descrição:** Monitor de sistema para KDE Plasma com arquitetura backend Rust + DBus + Plasmoid. O binário coleta métricas do sistema e o frontend em `plasma/` renderiza o widget compacto e o popup expandido.

## Stack

- **Linguagem(s):** Rust (edition 2021; README indica Rust 1.70+), QML
- **Frameworks:** Tokio, zbus, sysinfo, Plasma/Plasmoid API

## Gerenciamento de Dependências

- **Instalar tudo:** `(preencher manualmente)`
- **Adicionar pacote:** `(preencher manualmente)`
- **Remover pacote:** `(preencher manualmente)`

## Comandos Essenciais

- **Testes:** `make test`
- **Dev server:** `make dev` (backend DBus com cargo-watch)
- **Build:** `make build` ou `cargo build --release`
- **Backend DBus:** `make run-dbus`
- **JSON de teste:** `make run-json`
- **Fluxo KDE:** `make kde-dev` / `make kde-refresh`

## Estrutura de Diretórios

- **Código principal:** `src/`
- **Frontend KDE:** `plasma/`
- **Planos:** `.pi/plans/`
- **Testes:** `tests/` (não encontrado); testes unitários embutidos em `src/`

## Módulos

- **`src/lib.rs`** — Expõe a API compartilhada para coleta única de métricas, serialização em JSON e constantes do serviço DBus.
- **`src/main.rs`** — Entry point mínimo do binário; interpreta flags CLI (`--json`, `--dbus`, `--help`) e inicia o backend DBus por padrão.
- **`src/monitor.rs`** — Define os modelos serializáveis de CPU, memória, disco, rede e sensores e coleta snapshots do sistema via `sysinfo` e `/sys/class/hwmon`.
- **`src/dbus.rs`** — Implementa o backend DBus com `zbus`, mantendo um `SystemMonitor` compartilhado e expondo métricas em JSON para o frontend KDE.
- **`plasma/contents/ui/`** — Frontend do Plasmoid KDE, com representações compacta/expandida, componentes reutilizáveis e tabs.

## Arquitetura

- **Estilo:** Flat modular com separação backend/frontend
- **Descrição:** `monitor.rs` concentra a coleta e modelagem de métricas. `lib.rs` fornece funções reutilizáveis para coleta/serialização. `dbus.rs` publica o backend em DBus, e `main.rs` atua apenas como launcher do binário. O frontend do produto fica no Plasmoid em `plasma/`, que consulta o backend via DBus.

## Testes

- **Framework:** `cargo test` (testes unitários Rust)
- **Diretório:** `src/` com módulos `#[cfg(test)]`; `tests/` ⚠️ não encontrado
- **Executar todos:** `make test`
- **Validação adicional de runtime KDE:** `make kde-refresh` + teste manual do plasmoid no Plasma
- **Com cobertura:** `(preencher manualmente)`

## Convenções de Código

- **Tamanho máximo de função:** 150 linhas
- **Tamanho máximo de arquivo:** 700 linhas
- **Aninhamento máximo:** 3 níveis
- **Docstrings / comentários:** Português brasileiro
- **Identificadores (variáveis, funções, classes):** Inglês
- Rust: módulos separados por arquivo, `struct`s serializáveis para snapshots de métricas e `Result` para erros recuperáveis
- QML: preferir componentes pequenos e reutilizáveis em `plasma/contents/ui/components/`; tabs em `plasma/contents/ui/tabs/`

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
