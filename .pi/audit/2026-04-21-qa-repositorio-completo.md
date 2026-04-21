# Relatório de QA — Bugs, Segurança e Regras de Negócio
**Data:** 2026-04-21
**Escopo:** Repositório completo (`src/` + `plasma/`)
**Analista:** Agente QA

---

## 1. Resumo da Funcionalidade Analisada

Monitor de sistema para KDE Plasma. O binário Rust coleta métricas via `sysinfo`,
`/proc/stat`, `/proc/diskstats` e `/sys/class/hwmon`, expõe-as via DBus em JSON.
O Plasmoid QML consome o JSON e exibe 6 abas: CPU, RAM, Disk, Network, Sensors, System.

---

## 2. Resultado dos Testes Automáticos

```
cargo test:   10 passed, 0 failed
cargo clippy: No issues found
cargo build:  OK (dev + release)
```

---

## 3. Bugs e Condições de Erro

### Risco ALTO

Nenhum encontrado.

---

### Risco MÉDIO

- **[MÉDIO] `CpuTab.qml:33-62` — hero "Temperatura" exibe o sensor mais quente do sistema, não da CPU**

  `hottestTemp()` usa `metrics.sensors.hottest_temperature_celsius`, que é calculado
  sobre **todos** os sensores (GPU, NVMe, placa-mãe, etc.). No sistema de teste o
  hottest é `amdgpu: edge 51°C`, então a aba CPU mostra temperatura da GPU como se
  fosse da CPU.

  Risco: informação enganosa — o usuário acredita monitorar a CPU mas vê a GPU.

  Cenário: sistema com GPU mais quente que a CPU (comum).

  Sugestão: filtrar `metrics.sensors.temperatures` por `chip === "CPU"` ou
  `chipCategory(chip) === "CPU"` antes de calcular o máximo. Expor
  `hottest_cpu_temperature_celsius` no backend ou calcular em QML:
  ```javascript
  function hottestCpuTemp() {
      var cpuSensors = (metrics.sensors.temperatures || []).filter(function(s) {
          var n = (s.chip || "").toLowerCase();
          return n === "coretemp" || n === "k10temp" || n === "zenpower";
      });
      if (cpuSensors.length === 0) return null;
      return Math.max.apply(null, cpuSensors.map(function(s) {
          return s.temperature_celsius;
      }));
  }
  ```

---

- **[MÉDIO] `DiskTab.qml` — `primaryDisk()` recalculada 29 vezes por ciclo de render**

  A função cria e ordena um novo array a cada chamada. É invocada em:
  `title`, `subtitle`, `HeroMetric.value`, `HeroMetric.footnote`, `MetricBar.visible`,
  `MetricBar.label`, `MetricBar.value`, `MetricRow.value` — e mais referências indiretas.
  No total 29 chamadas por atualização (1500 ms).

  Risco: jank visual e CPU desnecessária em sistemas com muitos discos.

  Sugestão: cachear em uma propriedade computada:
  ```qml
  readonly property var cachedPrimaryDisk: primaryDisk()
  ```
  e substituir todas as referências por `root.cachedPrimaryDisk`.

---

- **[MÉDIO] `SensorsTab.qml:166,171,205` — `temperatureGroups()` chamada 3× por frame**

  Cada chamada itera e reagrupa toda a lista de sensores. Com 10 sensores e intervalo
  de 1500 ms é aceitável, mas cresce linearmente com o número de sensores.

  Sugestão: igual ao acima — usar `readonly property var cachedGroups: temperatureGroups()`.

---

- **[MÉDIO] `SensorsTab.qml:136-259` — `fansSorted()`, `voltagesSorted()`, `currentsSorted()`, `powersSorted()` chamadas 2–3× cada por frame**

  `fansSorted()` é chamada em `model:`, `subtitle:` e no `footnote` do hero (via
  `root.fansSorted().length`). Mesma situação para as outras três funções.

  Sugestão: `readonly property var cachedFans: fansSorted()` etc.

---

### Risco BAIXO

- **[BAIXO] `SensorsTab.qml:12` — `temperaturesSorted()` é código morto**

  A função é definida mas nunca chamada diretamente no template após a refatoração para
  `temperatureGroups()`. Permanece no arquivo sem uso.

  Sugestão: remover a função.

---

- **[BAIXO] `SensorsTab.qml:111` — string de desenvolvimento no subtitle do hero**

  ```qml
  subtitle: "Inspirado na referência de sensores do monitor"
  ```
  Texto de nota interna exibido em produção.

  Sugestão: substituir por algo informativo, ex.:
  `String(root.temperatureGroups().length) + " categorias de sensores"`.

---

- **[BAIXO] `collector.rs:370` — clone desnecessário de `hwmon_metrics.temperatures`**

  ```rust
  hwmon_metrics.temperatures.clone()
  ```
  `hwmon_metrics` é variável local; `temperatures` pode ser movido diretamente.

  Sugestão:
  ```rust
  hwmon_metrics.temperatures  // sem .clone()
  ```

---

- **[BAIXO] `collector.rs:232-236` — taxa de I/O assume janela de exatamente 200 ms**

  ```rust
  delta_read * SECTOR_BYTES * IO_INTERVALS_PER_SEC  // ×5 = assume 200 ms
  ```
  Se `tokio::time::sleep(200ms)` atrasar sob carga, a taxa será subestimada.
  Em sistemas muito carregados o erro pode ser perceptível.

  Sugestão: registrar o instante antes/depois do sleep com `std::time::Instant`
  e dividir pelo elapsed real em vez de usar o multiplicador fixo ×5.

---

- **[BAIXO] `main.qml` — `extractJsonPayload` não trata variantes com aspas duplas**

  ```javascript
  if (text.startsWith("('") && text.endsWith("',)")) { ... }
  ```
  Se o gdbus retornar `("...",)` (aspas duplas, variante de tipo `s`) a extração
  falha silenciosamente e o `JSON.parse` lança exceção capturada pelo try/catch,
  mostrando mensagem de erro genérica ao invés do payload.

  Sugestão: adicionar `|| (text.startsWith('("') && text.endsWith('",)'))` ao check,
  ajustando o `slice` para remover 2 e 3 caracteres respectivamente.

---

- **[BAIXO] Interfaces `lo` e virtuais (`tailscale0`) exibidas como DOWN**

  `read_interface_operstate` retorna `false` para `operstate = "unknown"`, que é o
  valor padrão de loopback e túneis virtuais. Essas interfaces aparecem com chip
  vermelho "DOWN" mesmo funcionando normalmente.

  Sugestão: tratar `unknown` como UP quando `bytes_received > 0` ou quando o nome
  for `lo`, ou ler `/sys/class/net/{name}/flags` para verificar o bit `IFF_UP`.

---

- **[BAIXO] `user% + system% + idle%` pode não somar 100% em VMs**

  `compute_cpu_percents` inclui `steal` no denominador mas não o exibe. Em VMs com
  steal time, a soma user + system + idle < 100%. O usuário pode interpretar como bug.

  Sugestão: expor `steal_percent` no modelo ou adicionar nota na UI.

---

## 4. Vulnerabilidades de Segurança

Nenhuma vulnerabilidade identificada.

- Sem SQL, sem endpoints HTTP, sem autenticação de usuário.
- DBus exposto apenas na sessão do usuário atual (sem acesso cross-user).
- `backendCommand` em `main.qml` é string literal — sem interpolação de input externo.
- Leituras de `/proc` e `/sys` são somente-leitura e sem input do usuário nos paths.

---

## 5. Falhas na Regra de Negócio

- **[BAIXO] `CpuTab` hero mostra temperatura errada** (duplicado do item MÉDIO acima):
  a regra de negócio implícita é "temperatura da CPU", mas a implementação mostra
  qualquer sensor — violando a expectativa do contexto da aba.

---

## 6. Resumo por Prioridade

| # | Severidade | Arquivo | Descrição |
|---|---|---|---|
| 1 | MÉDIO | `CpuTab.qml:33` | Temperatura no hero é do sensor mais quente global (pode ser GPU) |
| 2 | MÉDIO | `DiskTab.qml` | `primaryDisk()` recalculada 29× por frame |
| 3 | MÉDIO | `SensorsTab.qml:171` | `temperatureGroups()` recalculada 3× por frame |
| 4 | MÉDIO | `SensorsTab.qml:136` | `fansSorted()` e afins recalculados 2–3× por frame |
| 5 | BAIXO | `SensorsTab.qml:12` | `temperaturesSorted()` é código morto |
| 6 | BAIXO | `SensorsTab.qml:111` | Subtitle de desenvolvimento exposto em produção |
| 7 | BAIXO | `collector.rs:370` | Clone desnecessário de `hwmon_metrics.temperatures` |
| 8 | BAIXO | `collector.rs:232` | Taxa de I/O assume 200 ms fixo (pode subestimar sob carga) |
| 9 | BAIXO | `main.qml` | `extractJsonPayload` não trata variantes com aspas duplas |
| 10 | BAIXO | `collector.rs` | Interfaces `lo`/virtuais mostradas como DOWN |
| 11 | BAIXO | `collector.rs` | `user+system+idle` não soma 100% em VMs com steal time |
