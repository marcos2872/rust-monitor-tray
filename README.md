<p align="center">
  <img src="assets/icon.png" width="120" alt="Monitor Tray icon"/>
</p>

<h1 align="center">Monitor Tray</h1>

<p align="center">
  Monitor de sistema para <strong>KDE Plasma</strong> — backend Rust + DBus + Plasmoid
</p>

<p align="center">
  <img src="https://img.shields.io/badge/KDE_Plasma-6-3daee9?logo=kde&logoColor=white"/>
  <img src="https://img.shields.io/badge/Rust-1.70+-f74c00?logo=rust&logoColor=white"/>
  <img src="https://img.shields.io/badge/License-MIT-green"/>
</p>

---

## Instalação rápida

```bash
curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh
```

> **Requisitos:** Rust/Cargo, `kpackagetool6` (ou `kpackagetool5`), `systemctl --user`, `gdbus`

---

## O que é

Widget para o painel do KDE Plasma que exibe métricas do sistema em tempo real.
O binário Rust coleta os dados e os expõe via **DBus**; o Plasmoid QML consome
essa interface e renderiza a UI.

### Abas disponíveis

| Aba | Métricas |
|---|---|
| **CPU** | Uso total, user/system/idle/steal, load average, frequência, histórico, por núcleo |
| **RAM** | Usada/total, swap, histórico |
| **GPU** | Uso, VRAM, clocks, temperatura, potência, fan — AMD, NVIDIA e Intel |
| **Disk** | Uso por partição, I/O read/write em tempo real, histórico |
| **Network** | Download/upload instantâneo, histórico, status das interfaces |
| **Sensors** | Temperaturas por chip (CPU/GPU/NVMe), fans (RPM), tensão, corrente, potência |
| **System** | Hostname, OS, kernel, arquitetura, processos, load avg, resumo de hardware |

### Suporte a GPU

| Driver | Fonte | Dados |
|---|---|---|
| AMD (`amdgpu`) | `/sys/class/drm/` | Uso %, VRAM, clocks, temp, potência, fan RPM |
| NVIDIA | `nvidia-smi` | Uso %, VRAM, clocks, temp, potência |
| Intel (`i915`/`xe`) | `/sys/class/drm/` | Clock (kernel ≥ 5.16), temperatura |

---

## Instalação manual

```bash
git clone https://github.com/marcos2872/rust-monitor-tray.git
cd rust-monitor-tray
./install-kde.sh
```

O script:
1. Compila o backend Rust em release
2. Instala o binário em `~/.local/bin`
3. Instala o plasmoid via `kpackagetool`
4. Copia o ícone para `~/.local/share/icons/hicolor/` e atualiza o cache KDE
5. Cria e ativa o serviço `systemd --user` para o backend DBus

## Remoção

```bash
./uninstall-kde.sh
```

---

## Arquitetura

```
monitor-tray/
├── src/
│   ├── main.rs              # entry point (--dbus | --json | --help)
│   ├── lib.rs               # API pública: collect_metrics*
│   ├── dbus.rs              # serviço DBus (zbus)
│   └── monitor/
│       ├── models.rs        # structs serializáveis (CpuMetrics, GpuInfo…)
│       ├── collector.rs     # SystemMonitor: coleta e delta de métricas
│       ├── gpu.rs           # coleta GPU: AMD/NVIDIA/Intel
│       └── hwmon.rs         # leitura de /sys/class/hwmon
├── plasma/
│   ├── metadata.json
│   └── contents/ui/
│       ├── main.qml                  # polling DBus, histórico
│       ├── FullRepresentation.qml    # layout + tabs fixas no topo
│       ├── CompactRepresentation.qml # exibição no painel
│       ├── Theme.qml                 # paleta, espaçamentos, utilitários
│       ├── components/               # HeroMetric, HistoryChart, MetricBar…
│       └── tabs/                     # CpuTab, MemoryTab, GpuTab, DiskTab…
├── install.sh               # instalador one-liner (curl)
├── install-kde.sh           # instalação local completa
└── uninstall-kde.sh         # remoção completa
```

---

## Interface DBus

| Campo | Valor |
|---|---|
| Serviço | `com.monitortray.Backend` |
| Path | `/com/monitortray/Backend` |
| Métodos | `Ping`, `GetMetricsJson` |

```bash
# testar o backend manualmente
gdbus call --session \
  --dest com.monitortray.Backend \
  --object-path /com/monitortray/Backend \
  --method com.monitortray.Backend.GetMetricsJson
```

---

## Desenvolvimento

```bash
make test          # cargo test
make lint          # cargo clippy + qmllint
make kde-dev       # build + instala plasmoid + sobe backend DBus
make run-json      # imprime métricas em JSON (debug)
```

### Targets do Makefile

| Target | Descrição |
|---|---|
| `make build` | `cargo build --release` |
| `make test` | `cargo test` |
| `make lint` | clippy + qmllint |
| `make kde-refresh` | build + reinstala plasmoid |
| `make kde-dev` | `kde-refresh` + backend DBus |
| `make run-json` | amostra de métricas em JSON |
| `make run-dbus` | sobe o backend DBus |
| `make dev` | hot reload com `cargo-watch` |

---

## Dependências

### Rust
- Rust 1.70+, Cargo
- `rustup component add clippy`

### Sistema
- KDE Plasma 6 (ou 5)
- `kpackagetool6` / `kpackagetool5`
- `systemctl --user`
- `gdbus`
- Kernel com `/sys/class/hwmon` exposto (sensores)
- `nvidia-smi` no PATH para monitoramento NVIDIA

---

## Licença

MIT
