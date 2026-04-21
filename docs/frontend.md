# Frontend — Plasmoid QML

O frontend é um **Plasma Applet** escrito em QML. Ele consome o JSON do backend via DBus, mantém estado local para histórico e renderiza a interface compacta e o popup expandido.

---

## Estrutura de arquivos

```text
plasma/contents/ui/
├── main.qml                   # polling DBus, estado global e histórico
├── CompactRepresentation.qml  # resumo compacto no painel
├── FullRepresentation.qml     # popup completo com TabBar fixa
├── Theme.qml                  # design system: cores, espaçamentos e formatadores
├── components/                # componentes reutilizáveis
└── tabs/                      # conteúdo de cada aba
```

---

## Fluxo de dados no frontend

```mermaid
flowchart TD
    Watcher["DBusServiceWatcher"] --> StateCheck["serviço disponível?"]
    FastTimer["Timer 1500 ms expandido / 3000 ms compacto"] --> FastCall["asyncCall(FastMetricsJson)"]
    SlowTimer["Timer 4500 ms expandido"] --> SlowCall["asyncCall(SlowMetricsJson)"]
    StateCheck --> FastCall
    StateCheck --> SlowCall
    FastCall --> DBus["Session DBus"]
    SlowCall --> DBus
    DBus --> Extract["extractJsonPayload()"]
    Extract --> Parse["JSON.parse()"]
    Parse --> Apply["applyFastPayload() / applySlowPayload()"]
    Apply --> State["Estado segmentado por subsistema + histories"]
    State --> Full["FullRepresentation.qml"]
    State --> Compact["CompactRepresentation.qml"]
    Full --> Loader["Carregamento da aba ativa"]
    Loader --> Tabs["CpuTab, MemoryTab, GpuTab, DiskTab, NetworkTab, SensorsTab, SystemTab"]
```

---

## Estado global em `main.qml`

### Estado inicial relevante

O estado inicial do frontend já contempla os campos mais recentes do contrato:

- `networkMetrics.gateway_ip`
- `networkMetrics.gateway_latency_ms`
- `topProcesses`
- `gpuMetrics[*].fan_duty_percent`

> `sensorMetrics` no estado inicial ainda usa apenas campos essenciais; os campos opcionais novos chegam do backend no primeiro apply de payload.

### Polling e cliente DBus

`main.qml` usa um cliente DBus persistente do Plasma Workspace:

- `import org.kde.plasma.workspace.dbus 1.0 as DBus`
- `DBus.SessionBus.asyncCall(...)`
- `DBus.DBusServiceWatcher`

Regras atuais de atualização:

- timer rápido: `1500 ms` quando o popup está **expandido**;
- timer rápido: `3000 ms` quando o widget está apenas no modo **compacto**;
- timer lento: `4500 ms` quando o popup está **expandido**;
- `fastFetchInProgress` / `slowFetchInProgress` evitam chamadas sobrepostas;
- ao expandir o popup, o frontend dispara coleta imediata do caminho rápido;
- o primeiro fetch lento após abrir o popup usa pequeno debounce (`250 ms`) para reduzir burst no backend;
- se o backend ainda não expuser `FastMetricsJson` / `SlowMetricsJson`, o frontend recua automaticamente para `GetMetricsJson`.

### Caminho rápido em cache quente

O frontend continua chamando `FastMetricsJson`, mas esse método não mede mais CPU/disco sob demanda a cada request.

No desenho atual, o backend mantém um snapshot rápido em cache atualizado em background. Na prática, isso reduz bastante a latência percebida do fetch rápido no popup.

### Estado global segmentado

O frontend não usa mais um único `root.metrics` substituído por completo a cada resposta.

Em vez disso, o estado foi segmentado em propriedades por subsistema:

- `cpuMetrics`
- `memoryMetrics`
- `diskMetrics`
- `networkMetrics`
- `sensorMetrics`
- `gpuMetrics`
- `topProcesses`
- `systemInfoMetrics`
- `uptime`
- `loadAverage`

Isso reduz invalidação global de bindings e evita repassar um snapshot monolítico para toda a árvore QML.

### Histórico acumulado

| Propriedade | Conteúdo | Calculado em |
|---|---|---|
| `cpuHistory` | `cpu.usage_percent` | `applyFastPayload()` |
| `memoryHistory` | `memory.usage_percent` | `applyFastPayload()` |
| `networkDownloadRate` | delta local de bytes recebidos | `applyFastPayload()` |
| `networkUploadRate` | delta local de bytes enviados | `applyFastPayload()` |
| `networkDownloadHistory` | histórico de download | `applyFastPayload()` |
| `networkUploadHistory` | histórico de upload | `applyFastPayload()` |
| `diskReadRate` | `disk.total_read_bytes_per_sec` | `applyFastPayload()` |
| `diskWriteRate` | `disk.total_write_bytes_per_sec` | `applyFastPayload()` |
| `diskReadHistory` | histórico de leitura de disco | `applyFastPayload()` |
| `diskWriteHistory` | histórico de escrita de disco | `applyFastPayload()` |
| `gpuHistory` | `gpus[0].usage_percent` | `applySlowPayload()` |

O histórico é circular e usa:

- `expandedSampleIntervalMs = 1500`
- `compactSampleIntervalMs = 3000`
- `historyDurationMs = 5 * 60 * 1000`
- `historyLength = ceil(historyDurationMs / expandedSampleIntervalMs)`

Para reduzir trabalho em segundo plano, os históricos detalhados só são atualizados quando o popup está **expandido**.

Além disso, os históricos passaram a usar um buffer circular lógico, evitando `slice(0)` + `shift()` em cada amostra.

---

## Abas

| Índice | Aba | Arquivo | Props principais |
|---|---|---|---|
| 0 | CPU | `CpuTab.qml` | `cpuMetrics`, `sensorMetrics`, `history`, `historyDurationMs` |
| 1 | RAM | `MemoryTab.qml` | `memoryMetrics`, `history`, `historyDurationMs` |
| 2 | GPU | `GpuTab.qml` | `gpus`, `gpuHistory`, `historyDurationMs` |
| 3 | Disk | `DiskTab.qml` | `diskMetrics`, `diskReadHistory`, `diskWriteHistory`, `diskReadRate`, `diskWriteRate` |
| 4 | Network | `NetworkTab.qml` | `networkMetrics`, `networkSpeedTestStatus`, `downloadHistory`, `uploadHistory`, `downloadRate`, `uploadRate`, `historyDurationMs` |
| 5 | Sensors | `SensorsTab.qml` | `sensorMetrics` |
| 6 | System | `SystemTab.qml` | `systemInfo`, `topProcesses`, `uptime`, `loadAverage` |

### CPU — `CpuTab.qml`

- hero com temperatura, gauge de uso total e load de 1 minuto;
- usa **diretamente** `sensorMetrics.hottest_cpu_celsius` e `hottest_cpu_label`;
- histórico de uso da CPU;
- detalhes de `user`, `system`, `idle`, `steal` e uptime;
- grade por núcleo e load average.

### RAM — `MemoryTab.qml`

- hero com memória livre, uso total e swap;
- histórico de uso;
- detalhes de RAM usada, disponível e swap.

### GPU — `GpuTab.qml`

- suporta GPU primária com visual completo e GPUs secundárias em cards compactos;
- hero com temperatura, gauge de uso e potência;
- histórico de uso da GPU primária;
- seção de VRAM e clocks;
- exibe `fan_rpm` e, quando disponível, `fan_duty_percent` no hero e em details.

### Disk — `DiskTab.qml`

- uso do disco principal com barra de progresso;
- atividade de leitura e escrita em tempo real;
- histórico de I/O;
- partições secundárias em seções compactas.

### Network — `NetworkTab.qml`

- hero com download e upload instantâneos;
- histórico separado de download e upload;
- card de **teste de velocidade manual** com botão de iniciar/cancelar;
- details com as interfaces mais ativas;
- exibe `gateway_ip` e `gateway_latency_ms` quando disponíveis;
- usa cor dinâmica para latência: verde, amarelo ou vermelho conforme o valor.

O speed test usa um fluxo separado do polling normal:

- `StartNetworkSpeedTest` para iniciar;
- `GetNetworkSpeedTestStatusJson` para consultar estado;
- `CancelNetworkSpeedTest` para cancelar;
- timer dedicado de `1000 ms` no frontend apenas enquanto o teste estiver em execução.

### Sensors — `SensorsTab.qml`

- hero com estado geral, pico de temperatura e ventilação;
- temperaturas por chip;
- fans com RPM e duty cycle quando disponível;
- listas de tensão, corrente e potência.

### System — `SystemTab.qml`

- hero com uptime, total de processos e arquitetura;
- card do sistema operacional;
- card de load average;
- lista dos **15 processos com maior uso de CPU**;
- a antiga visão resumida de recursos foi substituída pelo ranking de processos.

---

## Design system — `Theme.qml`

### Paleta de cores

| Propriedade | Uso |
|---|---|
| `cpuColor` | CPU, download |
| `memoryColor` | RAM, VRAM |
| `swapColor` | Swap |
| `diskColor` | Disco |
| `networkColor` | Rede |
| `gpuColor` | GPU |
| `systemColor` | Informações gerais |
| `successColor` | Status saudável / ativo |
| `warningColor` | Estado intermediário |
| `dangerColor` | Estado crítico |

### Utilitários usados pelas abas

| Função | Descrição |
|---|---|
| `theme.fmtUptime(s)` | Formata uptime como `Xd Yh Zm` |
| `theme.fmtBytes(v)` | Formata bytes com unidade automática |
| `theme.fmtRate(v)` | Formata taxa em bytes por segundo |
| `theme.fmtOne(v)` | Uma casa decimal com fallback seguro |

---

## Resumo técnico

O frontend mantém apenas lógica de apresentação e derivação leve. Com as mudanças recentes, houve uma redução de lógica duplicada e de overhead no QML:

- a temperatura principal da aba CPU agora vem pronta do backend;
- a latência do gateway chega pronta no payload de rede;
- a aba System consome `top_processes` diretamente do backend;
- o detalhe de `fan_duty_percent` da GPU passou a ser exibido sem necessidade de cálculo local;
- o polling deixou de usar subprocesso `gdbus call` e passou a usar cliente DBus persistente assíncrono;
- o caminho quente passou a consumir payload DBus reduzido (`FastMetricsJson`);
- o caminho quente passou a ler um snapshot rápido em cache mantido pelo backend;
- o estado QML deixou de ser substituído como um objeto monolítico;
- os históricos deixaram de copiar arrays inteiros em cada amostra;
- o teste manual de velocidade foi isolado em um fluxo DBus próprio, fora do polling contínuo.
