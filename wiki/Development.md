# Desenvolvimento

---

## Pré-requisitos

```bash
# Rust e clippy
rustup update stable
rustup component add clippy

# KDE Plasma SDK
# Fedora:
sudo dnf install plasma-sdk

# Ubuntu/Debian:
sudo apt install plasma-sdk
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
| `make qml-lint` | Valida apenas os arquivos QML |
| `make run-json` | Imprime uma amostra de métricas em JSON |
| `make run-dbus` | Sobe o backend DBus |
| `make kde-refresh` | Build + reinstala o plasmoid |
| `make kde-dev` | `kde-refresh` + backend DBus |
| `make dev` | Hot reload do backend com `cargo-watch` |

---

## Testes

```bash
make test
# ou
cargo test
```

Os testes unitários ficam em `src/monitor/mod.rs` e `src/monitor/collector.rs` dentro de módulos `#[cfg(test)]`.

---

## Lint

```bash
make lint
# equivale a:
cargo clippy --all-targets --all-features -- -D warnings
qmllint plasma/contents/ui/**/*.qml
```

---

## Estrutura do projeto

```
src/
├── main.rs              # Entry point: --dbus | --json | --help
├── lib.rs               # API pública: collect_metrics*
├── dbus.rs              # Serviço DBus (zbus)
└── monitor/
    ├── models.rs        # Structs serializáveis do payload
    ├── collector.rs     # SystemMonitor: delta CPU/Disk, cache GPU
    ├── gpu.rs           # Coleta GPU: AMD/NVIDIA/Intel
    └── hwmon.rs         # Leitura de /sys/class/hwmon

plasma/contents/ui/
├── main.qml             # Orchestração: polling DBus, histórico
├── FullRepresentation.qml  # Layout: header fixo + ScrollView
├── CompactRepresentation.qml  # Exibição no painel
├── Theme.qml            # Paleta, espaçamentos, funções utilitárias
├── components/          # MetricCard, HistoryChart, RingGauge…
└── tabs/                # CpuTab, GpuTab, DiskTab…
```

---

## Publicar uma release

```bash
git tag v1.0.0
git push origin v1.0.0
```

O GitHub Actions compila, testa e publica automaticamente. Veja [RELEASE.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/RELEASE.md) para detalhes.
