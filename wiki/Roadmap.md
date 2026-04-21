# Roadmap

---

## ✅ Implementado

### Backend
- [x] Coleta de CPU: uso total, user/system/idle/steal via `/proc/stat`
- [x] Coleta de RAM e swap via sysinfo
- [x] Coleta de disco: espaço e I/O em tempo real via `/proc/diskstats`
- [x] Coleta de rede: bytes acumulados, status das interfaces, gateway padrão e latência
- [x] Sensores via `/sys/class/hwmon`: temperaturas, fans, tensão, corrente e potência
- [x] Campos dedicados para temperatura principal de CPU e GPU em `SensorMetrics`
- [x] Top processos por CPU com uso normalizado para `0–100%` do sistema total
- [x] GPU AMD via sysfs: uso%, VRAM, clocks, temperatura, potência, fan RPM e duty%
- [x] GPU NVIDIA via `nvidia-smi`
- [x] GPU Intel via sysfs: clock e temperatura quando disponível
- [x] Informações do sistema: hostname, OS, kernel, arquitetura e contagem de processos
- [x] Serviço `systemd --user` para o backend DBus

### Frontend QML
- [x] 7 abas: CPU, RAM, GPU, Disk, Network, Sensors e System
- [x] TabBar fixa no topo, scroll apenas no conteúdo
- [x] Histórico de 5 minutos para CPU, RAM, GPU, disco e rede
- [x] Hero da aba CPU usando temperatura principal já derivada no backend
- [x] Aba GPU exibindo fan RPM e duty% quando disponível
- [x] Aba Network exibindo gateway padrão e latência
- [x] Aba System exibindo top processos em vez do card de recursos resumidos
- [x] Design system centralizado em `Theme.qml`
- [x] Ícone personalizado no catálogo de widgets

### CI/CD e distribuição
- [x] GitHub Actions: release automática ao criar tag `v*`
- [x] Instalador one-liner via `curl`

---

## 🔄 Planejado

### Frontend QML
- [ ] Configurações por aba (intervalo de atualização, janela de histórico)
- [ ] Gráfico de rede bidirecional com visual espelhado
- [ ] Modo compacto configurável (quais métricas mostrar no painel)
- [ ] Suporte a tema claro do Plasma

### Backend e produto
- [ ] Filtros ou ordenações alternativas para a lista de processos
- [ ] Mais metadados de GPU Intel, se o kernel/driver expuserem caminhos estáveis
- [ ] Configuração opcional para intervalo da medição de latência

### Distribuição
- [ ] Pacote para a KDE Store
- [ ] Build para `aarch64` (Raspberry Pi, ARM Linux)
- [ ] Flatpak ou script de instalação para distros sem `systemd`

---

## ❌ Fora de escopo

- Suporte a macOS ou Windows
- Monitoramento de temperatura via IPMI
- Interface web ou servidor HTTP
