# AGENTS.md

> Arquivo gerado por `/init` com análise automática. Edite manualmente para ajustar convenções.

## Projeto

- **Nome:** monitor-tray
- **Descrição:** Monitor de sistema para KDE Plasma — CPU, RAM, GPU, Disco, Rede, Sensores e Sistema em um widget de painel.

## Stack

- **Linguagem(s):** Rust (edition 2021; README indica Rust 1.70+), QML
- **Frameworks:** Tokio, zbus, sysinfo

## Gerenciamento de Dependências

- **Instalar tudo:** `(preencher manualmente)`
- **Adicionar pacote:** `(preencher manualmente)`
- **Remover pacote:** `(preencher manualmente)`

## Comandos Essenciais

- **Testes:** `make test`
- **Lint:** `make lint`
- **QML lint:** `make qml-lint`
- **Dev server:** `make dev`
- **Build:** `make build`
- **Backend DBus:** `make run-dbus`
- **JSON de teste:** `make run-json`
- **Fluxo KDE:** `make kde-dev` / `make kde-refresh`

## Estrutura de Diretórios

- **Código principal:** `src/`
- **Testes:** `tests/` (não encontrado)

## Módulos

- **`src/lib.rs`** — Expõe a API compartilhada para coleta de métricas, serialização em JSON e controle do teste de velocidade.
- **`src/main.rs`** — Entry point do binário; interpreta `--dbus`, `--json` e `--help` e inicia o backend DBus por padrão.
- **`src/dbus.rs`** — Publica a interface DBus `com.monitortray.Backend` e expõe métodos assíncronos para métricas e speed test.
- **`src/monitor/`** — Concentra a coleta e modelagem de métricas do sistema, separando métricas rápidas e lentas.
- **`src/speedtest.rs`** — Gerencia o teste manual de velocidade de rede com fallback entre `speedtest` e `speedtest-cli`.

## Arquitetura

- **Estilo:** Backend DBus modular com frontend Plasmoid KDE
- **Descrição:** `main.rs` atua como launcher CLI e delega ao serviço DBus em `dbus.rs`. `lib.rs` expõe funções reutilizáveis, `monitor/` coleta snapshots do sistema com cache e ciclos de atualização, e `speedtest.rs` executa testes de rede assíncronos consumidos pelo frontend Plasma.

## Testes

- **Framework:** `cargo test`
- **Diretório:** `tests/` ⚠️ não encontrado; testes unitários embutidos em `src/`
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
