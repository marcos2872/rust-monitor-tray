# Arquitetura

---

## Visão geral

O Monitor Tray separa **coleta de dados** (backend Rust) de **apresentação** (frontend QML) por meio de uma interface **Session DBus**.

```
┌─────────────────────────────────────────────┐
│              Monitor Tray                   │
│                                             │
│  ┌────────────────┐   DBus    ┌──────────┐  │
│  │  Backend Rust  │◄────────►│ Plasmoid │  │
│  │  monitor-tray  │           │   QML    │  │
│  └───────┬────────┘           └──────────┘  │
│          │                                  │
└──────────┼──────────────────────────────────┘
           │
    ┌──────▼──────┐   ┌──────────────┐
    │ Kernel Linux │   │  nvidia-smi  │
    │ /sys  /proc  │   │ (NVIDIA GPU) │
    └─────────────┘   └──────────────┘
```

**Backend Rust** coleta métricas a cada ciclo de ~200 ms e expõe `GetMetricsJson()` via DBus.  
**Plasmoid QML** chama esse método a cada 1500 ms, acumula histórico e renderiza 7 abas.

---

## Ciclo de atualização

```
Timer 1500ms (QML)
    │
    ▼ gdbus call GetMetricsJson
Backend Rust
    ├── /proc/stat snapshot (antes)
    ├── /proc/diskstats snapshot (antes)
    ├── sysinfo refresh #1
    ├── sleep 200ms
    ├── sysinfo refresh #2
    ├── /proc/stat snapshot (depois) → compute_cpu_percents()
    ├── /proc/diskstats snapshot (depois) → compute_disk_io_rates()
    └── collect_gpu_metrics() async
            ├── AMD: /sys/class/drm/cardN/device/
            ├── Intel: /sys/class/drm/cardN/gt/
            └── NVIDIA: subprocess nvidia-smi --format=csv
    │
    ▼ JSON → DBus → QML → applyMetrics() → re-render
```

---

## Módulos principais

| Módulo | Responsabilidade |
|---|---|
| `src/monitor/collector.rs` | `SystemMonitor`: estado, delta CPU/Disk, cache GPU |
| `src/monitor/gpu.rs` | Detecção e coleta por vendor (AMD/NVIDIA/Intel) |
| `src/monitor/hwmon.rs` | Sensores via `/sys/class/hwmon` |
| `src/monitor/models.rs` | Structs do payload JSON |
| `src/dbus.rs` | Serviço DBus com `zbus` |
| `plasma/…/main.qml` | Polling, histórico, estado global |
| `plasma/…/Theme.qml` | Design system: paleta + funções utilitárias |

---

## Documentação técnica detalhada

A documentação completa fica em [`docs/`](https://github.com/marcos2872/rust-monitor-tray/tree/main/docs) no repositório:

| Arquivo | Conteúdo |
|---|---|
| [architecture.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/architecture.md) | Diagramas C4, fluxo de dados, inventário |
| [backend.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/backend.md) | Interface DBus, ciclo de coleta, testes |
| [models.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/models.md) | Referência completa dos structs JSON |
| [frontend.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/frontend.md) | Abas, histórico, design system |
| [components.md](https://github.com/marcos2872/rust-monitor-tray/blob/main/docs/components.md) | Props de todos os componentes QML |
