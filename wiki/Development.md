# Desenvolvimento

---

## Pré-requisitos

```bash
# Rust e clippy
rustup update stable
rustup component add clippy

# KDE Plasma SDK
# Fedora:
sudo dnf install plasma-sdk qt6-qtdeclarative

# Ubuntu/Debian:
sudo apt install plasma-sdk qt6-declarative-dev-tools
```

---

## Clonar e configurar

```bash
git clone https://github.com/marcos2872/rust-monitor-tray.git
cd rust-monitor-tray
```

---

## Fluxo de desenvolvimento

```bash
# Instala o plasmoid, sobe o backend DBus e fica assistindo mudanças
make kde-dev

# Apenas reinstala o plasmoid (sem subir o backend)
make kde-refresh

# Sobe o backend DBus manualmente
make run-dbus

# Testa o payload JSON sem o Plasma
make run-json
```

---

## Targets do Makefile

| Target | Descrição |
|---|---|
| `make build` | `cargo build --release` |
| `make test` | `cargo test` |
| `make lint` | `cargo clippy` + `qmllint` |
| `make qml-lint` | Valida apenas os arquivos QML (`qmllint`) |
| `make run-json` | Imprime uma amostra de métricas em JSON |
| `make run-dbus` | Sobe o backend DBus |
| `make kde-refresh` | Build + reinstala o plasmoid |
| `make kde-dev` | `kde-refresh` + backend DBus |
| `make dev` | Hot reload do backend com `cargo-watch` |

---

## Onde encontrar a documentação certa

| Se você precisa... | Leia |
|---|---|
| entender a visão geral do projeto | `wiki/` |
| alterar o payload JSON | `docs/models.md` |
| alterar a coleta backend | `docs/backend.md` |
| alterar o fluxo ou as abas QML | `docs/frontend.md` |
| entender módulos e dependências | `docs/architecture.md` |

---

## Convenção prática para mudanças

Sempre que alterar um desses pontos, atualize a referência técnica correspondente:

- campos novos em `SystemMetrics` ou structs filhas → `docs/models.md`
- nova fonte Linux, novo subprocesso ou mudança no ciclo → `docs/backend.md`
- mudança de comportamento em aba/componentes/estado QML → `docs/frontend.md`
- mudança estrutural entre backend, DBus e frontend → `docs/architecture.md`
- mudança no cache quente de `FastMetricsJson` ou no fluxo rápido/lento → `docs/backend.md` e `docs/frontend.md`

---

## Estrutura do projeto

```text
src/
├── main.rs              # Entry point: --dbus | --json | --help
├── lib.rs               # API pública: collect_metrics*
├── dbus.rs              # Serviço DBus (zbus)
└── monitor/
    ├── models.rs        # Structs serializáveis do payload
    ├── collector.rs     # SystemMonitor: deltas, caches e snapshot final
    ├── gpu.rs           # Coleta GPU: AMD/NVIDIA/Intel
    └── hwmon.rs         # Leitura de /sys/class/hwmon

plasma/contents/ui/
├── main.qml                 # cliente DBus persistente, polling rápido/lento, debounce do caminho lento e estado segmentado
├── FullRepresentation.qml   # layout expandido
├── CompactRepresentation.qml# resumo no painel
├── Theme.qml                # paleta e formatadores
├── components/              # MetricCard, HistoryChart, RingGauge…
└── tabs/                    # CpuTab, MemoryTab, GpuTab, DiskTab...
```

---

## Testes e validação

```bash
make test
make lint
```

Os testes unitários ficam em `src/monitor/mod.rs` e `src/monitor/collector.rs`.

Quando alterar métricas recentes, vale validar manualmente também:

- polling DBus rápido no popup expandido e no modo compacto;
- `FastMetricsJson` servido do cache quente do backend;
- polling DBus lento para sensores/GPU/processos quando expandido;
- debounce antes do primeiro fetch lento ao abrir o popup;
- fallback para `GetMetricsJson` quando o backend ainda estiver em versão antiga;
- timer separado de status para o teste manual de velocidade na aba `Network`;
- reconexão automática quando o backend DBus sobe/para;

- `top_processes` na aba **System**;
- `gateway_ip` e `gateway_latency_ms` na aba **Network**;
- teste manual de velocidade na aba **Network** (requer `speedtest` ou `speedtest-cli` instalado no sistema);
- `fan_duty_percent` na aba **GPU**;
- `hottest_cpu_celsius` no hero da aba **CPU**.

---

## Publicar uma release

```bash
git tag v0.1.1
git push origin v0.1.1
```

O GitHub Actions compila, testa e publica automaticamente. Veja [RELEASE.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/RELEASE.md) para detalhes.
