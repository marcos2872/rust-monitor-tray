# Arquitetura — Monitor Tray

## Visão Geral

O Monitor Tray é um widget para **KDE Plasma** que exibe métricas do sistema em tempo real. A arquitetura separa coleta de dados (backend Rust) de apresentação (frontend QML) por meio de uma interface **DBus**.

---

## C4 — Contexto do Sistema

```mermaid
graph TD
    User["👤 Usuário\n[KDE Plasma Desktop]"]
    MonitorTray["🖥️ Monitor Tray\n[Widget KDE Plasma]"]
    Kernel["🐧 Kernel Linux\n[/sys, /proc, hwmon]"]
    NvidiaSmi["⚙️ nvidia-smi\n[Ferramenta NVIDIA]"]

    User -->|"Adiciona widget ao painel\nVisualiza métricas"| MonitorTray
    MonitorTray -->|"Lê sensores, CPU, disco, rede\nvia sysfs e sysinfo"| Kernel
    MonitorTray -->|"Coleta métricas GPU NVIDIA\nsubprocess"| NvidiaSmi
```

---

## C4 — Containers

```mermaid
graph LR
    subgraph MonitorTray["Monitor Tray"]
        Backend["🦀 Backend Rust\n[Binário: monitor-tray]\nColeta métricas e expõe via DBus"]
        Plasmoid["🎨 Plasmoid QML\n[KDE Plasma Widget]\nUI com 7 tabs e gráficos"]
    end

    Kernel["🐧 Kernel Linux\n[/sys/class/hwmon\n/proc/stat\n/proc/diskstats]"]
    NvidiaSmi["⚙️ nvidia-smi\n[Subprocess]"]
    DBus["🔌 Session DBus\n[com.monitortray.Backend]"]

    Backend -->|"Lê arquivos sysfs/proc"| Kernel
    Backend -->|"Executa subprocess async"| NvidiaSmi
    Backend -->|"Publica GetMetricsJson()"| DBus
    Plasmoid -->|"Chama GetMetricsJson()\na cada 1500 ms via gdbus"| DBus
```

---

## C4 — Componentes do Backend

```mermaid
graph TD
    main["main.rs\nEntry point CLI\n--dbus / --json / --help"]
    lib["lib.rs\ncollect_metrics*\nAPI pública"]
    dbus["dbus.rs\nMetricsBackend\nzbus interface"]
    collector["collector.rs\nSystemMonitor\nDelta CPU/Disk"]
    gpu["gpu.rs\ncollect_gpu_metrics()\nAMD/NVIDIA/Intel"]
    hwmon["hwmon.rs\ncollect_hwmon_metrics()\ntemp/fan/volt/curr/power"]
    models["models.rs\nSystemMetrics\nstructs serializáveis"]

    main --> lib
    lib --> dbus
    lib --> collector
    collector --> gpu
    collector --> hwmon
    collector --> models
    gpu --> models
    hwmon --> models
```

---

## Fluxo de Dados

```mermaid
sequenceDiagram
    participant P as Plasmoid QML
    participant D as Session DBus
    participant B as Backend Rust
    participant S as Sistema (sysfs/proc)

    loop a cada 1500 ms
        P->>D: gdbus call GetMetricsJson
        D->>B: GetMetricsJson()
        B->>S: refresh sysinfo + /proc/stat + /proc/diskstats + /sys/class/hwmon
        B->>B: compute_cpu_percents() — delta 200ms
        B->>B: compute_disk_io_rates() — delta 200ms
        B->>B: collect_gpu_metrics() — AMD sysfs ou nvidia-smi
        B-->>D: JSON string (SystemMetrics)
        D-->>P: ('{ "cpu": {...}, "gpus": [...], ... }',)
        P->>P: applyMetrics() — acumula histórico, calcula taxas de rede
        P->>P: re-render tabs ativas
    end
```

---

## Inventário de Módulos

| Módulo | Tipo | Responsabilidade |
|---|---|---|
| `src/main.rs` | Entry point | Parse de flags CLI; delega para `lib.rs` |
| `src/lib.rs` | API pública | Funções `collect_metrics*`; constantes DBus |
| `src/dbus.rs` | Serviço DBus | Expõe `GetMetricsJson` via `zbus` com `Mutex<SystemMonitor>` |
| `src/monitor/models.rs` | Modelos | Structs `#[derive(Serialize, Deserialize)]` para todo o payload |
| `src/monitor/collector.rs` | Coleta | `SystemMonitor`: delta de CPU/Disk, taxas de I/O, cache de GPU |
| `src/monitor/gpu.rs` | Coleta GPU | AMD via sysfs, NVIDIA via `nvidia-smi`, Intel via sysfs (limitado) |
| `src/monitor/hwmon.rs` | Sensores | Leitura de `temp/fan/in/curr/power` em `/sys/class/hwmon` |
| `plasma/…/main.qml` | Orchestração | Polling DBus, acúmulo de histórico, estado global |
| `plasma/…/FullRepresentation.qml` | Layout | TabBar fixa + ScrollView do conteúdo |
| `plasma/…/Theme.qml` | Design system | Paleta, espaçamentos, funções utilitárias (`fmtBytes`, `fmtUptime`…) |
| `plasma/…/components/` | UI reutilizável | `MetricCard`, `HeroMetric`, `HistoryChart`, `RingGauge`… |
| `plasma/…/tabs/` | Conteúdo das abas | `CpuTab`, `RamTab`, `GpuTab`, `DiskTab`, `NetworkTab`, `SensorsTab`, `SystemTab` |

---

## Decisões de Arquitetura

- [0001 — Backend Rust com interface DBus](adr/0001-backend-rust-dbus.md)
- [0002 — Monitoramento de GPU via sysfs e nvidia-smi](adr/0002-gpu-sysfs-nvidia-smi.md)
- [0003 — Serviço systemd do usuário para o backend](adr/0003-systemd-user-service.md)
