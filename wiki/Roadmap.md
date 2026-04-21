# Roadmap

---

## ✅ Implementado (v0.1.0)

### Backend
- [x] Coleta de CPU: uso total, user/system/idle/steal via `/proc/stat`
- [x] Coleta de RAM e swap via sysinfo
- [x] Coleta de disco: espaço e I/O em tempo real via `/proc/diskstats`
- [x] Coleta de rede: bytes acumulados e status das interfaces
- [x] Sensores: temperaturas agrupadas por chip, fans (RPM + duty%), tensão, corrente, potência via `/sys/class/hwmon`
- [x] GPU AMD via sysfs: uso%, VRAM, clocks, temperatura, potência, fan RPM
- [x] GPU NVIDIA: múltiplas GPUs suportadas — `nvidia-smi` retorna todas (uma linha por GPU no CSV)
- [x] GPU Intel via sysfs: clock e temperatura (limitado, kernel ≥ 5.16)
- [x] Informações do sistema: hostname, OS, kernel, arquitetura, contagem de processos
- [x] Serviço `systemd --user` para o backend DBus
- [x] Fan duty% AMD disponível em `SensorMetrics.fans` via hwmon (visível na aba Sensors)

### Frontend QML
- [x] 7 abas: CPU, RAM, GPU, Disk, Network, Sensors, System
- [x] TabBar fixa no topo, scroll apenas no conteúdo
- [x] Histórico de 5 minutos: CPU, RAM, GPU, Disk I/O, Download, Upload
- [x] Temperatura da CPU filtrada por chip (`coretemp`/`k10temp`) no hero da aba CPU
- [x] Temperatura da GPU disponível em `GpuInfo.temperature_celsius` na aba GPU
- [x] Design system centralizado no `Theme.qml`
- [x] Ícone personalizado no catálogo de widgets

### CI/CD
- [x] GitHub Actions: release automática ao criar tag `v*`
- [x] Instalador one-liner via `curl`

---

## 🔄 Planejado

### Backend
- [ ] Percentual de uso por processo (top N processos por CPU e RAM)
- [ ] Latência de rede (ping ao gateway)
- [ ] Campos dedicados no modelo: `hottest_cpu_celsius` e `hottest_gpu_celsius` em `SensorMetrics`
- [ ] `GpuInfo.fan_duty_percent` para AMD (duty% da GPU especificamente, além do RPM já existente)

### Frontend QML
- [ ] Configurações por aba (intervalo de atualização, janela de histórico)
- [ ] Aba de processos: top N por CPU e RAM
- [ ] Gráfico de rede bidirecional (upload/download espelhados)
- [ ] Modo compacto configurável (quais métricas mostrar no painel)
- [ ] Suporte a tema claro do Plasma

### Distribuição
- [ ] Pacote para a KDE Store
- [ ] Build para `aarch64` (Raspberry Pi, ARM Linux)
- [ ] Flatpak ou script de instalação para distros sem `systemd`

---

## ❌ Fora de escopo

- Suporte a macOS ou Windows
- Monitoramento de temperatura via IPMI
- Interface web ou servidor HTTP
