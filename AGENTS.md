# AGENTS.md

> Arquivo gerado por `/init` com análise automática. Edite manualmente para ajustar convenções.

## Projeto

- **Nome:** monitor-tray
- **Descrição:** Aplicativo de monitoramento de sistema para Linux que exibe métricas em tempo real na system tray. Inclui UI legada com GTK/AppIndicator e backend DBus para um Plasmoid KDE.

## Stack

- **Linguagem(s):** Rust (edition 2021; README indica Rust 1.70+)
- **Frameworks:** GTK, libappindicator, Tokio, zbus

## Gerenciamento de Dependências

- **Instalar tudo:** `(preencher manualmente)`
- **Adicionar pacote:** `(preencher manualmente)`
- **Remover pacote:** `(preencher manualmente)`

## Comandos Essenciais

- **Testes:** `make test`
- **Dev server:** `make dev`
- **Build:** `cargo build --release`

## Estrutura de Diretórios

- **Código principal:** `src/`
- **Testes:** `tests/` (não encontrado); testes unitários embutidos em `src/`

## Módulos

- **`src/lib.rs`** — Expõe a API compartilhada para coleta única de métricas, serialização em JSON e constantes do serviço DBus.
- **`src/main.rs`** — Inicializa a aplicação GTK/AppIndicator, interpreta flags CLI (`--json`, `--dbus`), cria o menu da tray e atualiza a interface periodicamente.
- **`src/monitor.rs`** — Define os modelos serializáveis de CPU, memória, disco, rede e sensores e coleta snapshots do sistema via `sysinfo` e `/sys/class/hwmon`.
- **`src/dbus.rs`** — Implementa o backend DBus com `zbus`, mantendo um `SystemMonitor` compartilhado e expondo métricas em JSON para o frontend KDE.

## Arquitetura

- **Estilo:** Flat modular
- **Descrição:** `monitor.rs` concentra a coleta e modelagem de métricas. `lib.rs` fornece funções reutilizáveis para coleta/serialização, enquanto `main.rs` usa essas rotinas para a UI de tray e `dbus.rs` publica o mesmo backend em DBus para consumo do Plasmoid KDE.

## Testes

- **Framework:** `cargo test` (testes unitários Rust)
- **Diretório:** `src/` com módulos `#[cfg(test)]`; `tests/` ⚠️ não encontrado
- **Executar todos:** `make test`
- **Com cobertura:** `(preencher manualmente)`

## Convenções de Código

- **Tamanho máximo de função:** 150 linhas
- **Tamanho máximo de arquivo:** 700 linhas
- **Aninhamento máximo:** 3 níveis
- **Docstrings / comentários:** Português brasileiro
- **Identificadores (variáveis, funções, classes):** Inglês
- Rust: módulos separados por arquivo, `struct`s serializáveis para snapshots de métricas e `Result` para erros recuperáveis

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
| `test`    | Cria e mantém testes automatizados             | escrita em tests/      |
| `doc`     | Cria documentação técnica em `docs/`           | escrita em docs/       |
