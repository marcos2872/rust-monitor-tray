# Relatório de Qualidade de Código
**Data:** 2026-04-21
**Escopo:** Repositório completo
**Stack detectada:** Rust (edition 2021) + QML (Plasma/Plasmoid)

---

## Ferramentas Automáticas

### cargo test
```
10 passed, 0 failed
```

### cargo clippy --all-targets --all-features -- -D warnings
```
Nenhum problema encontrado
```

---

## Arquitetura

Nenhum problema encontrado. As fronteiras estão corretas:

- `models.rs` → sem dependências internas
- `hwmon.rs` → importa apenas tipos de `super` (models)
- `collector.rs` → importa `super::hwmon` e `super` (models)
- `dbus.rs` → importa apenas `crate` (lib pública)
- Frontend QML → nunca acessa o backend diretamente; tudo via `metrics` prop

---

## Rust — Estilo e Convenções

### Tamanho de arquivos e funções

Todos os arquivos estão dentro do limite de 700 linhas e todas as funções dentro
de 150 linhas. Nenhum problema encontrado.

### Aninhamento

- **[AVISO] `collector.rs:212` — nível 4 no loop de I/O de disco**

  ```rust
  for (name, (sr_after, sw_after)) in &disk_io_after {
      if let Some((sr_before, sw_before)) = disk_io_before.get(name) { // nível 4
          ...
      }
  }
  ```
  Viola o limite de 3 níveis do AGENTS.md.

  Sugestão: extrair para função `compute_disk_rates(before, after, elapsed) -> HashMap<String, u64>`.

- **[AVISO] `collector.rs:278` — nível 4 dentro do `.map()` em `get_disk_metrics`**

  O closure passado a `.map()` já está em nível 3 (impl → método → closure);
  o `if total_space > 0.0` dentro dele atinge nível 4.

  Sugestão: extrair a construção de `DiskInfo` para `fn build_disk_info(disk, read_rate, write_rate) -> DiskInfo`.

### `unwrap()` / `expect()` em produção

Nenhum encontrado fora dos testes.

### Comentários em português brasileiro

- **[AVISO] `hwmon.rs` — zero comentários em todas as funções privadas**

  Funções como `read_trimmed`, `read_scaled_f32`, `prettify_identifier`,
  `hwmon_chip_name`, `hwmon_sensor_label` e o loop principal de
  `collect_hwmon_metrics_from_path` (116 linhas) não têm nenhum comentário.
  O AGENTS.md exige comentários em PT-BR.

### Testes

- **[AVISO] `test_collect_hwmon_metrics_reads_fans_voltage_current_and_power` —
  não cobre leitura de temperatura**

  O teste de hwmon cria fixtures de `fan1_input`, `in0_input`, `curr1_input` e
  `power1_input`, mas **não testa `temp1_input`**, que foi adicionado recentemente
  e é a principal mudança nesse módulo.

  Sugestão: adicionar `temp1_input` e `temp1_label` ao fixture e verificar
  `metrics.temperatures`.

- **[AVISO] Funções privadas críticas sem teste unitário**

  `compute_cpu_percents`, `read_diskstats`, `read_cpu_stat_raw` e `device_basename`
  não têm testes. São funções puras ou com comportamento determinístico — ideais
  para testes unitários.

- **[AVISO] `test_get_all_metrics_returns_non_negative_snapshot` — não verifica
  `SystemInfo`**

  `system_info` foi adicionado mas o teste não asserta `hostname`, `os_name`,
  `kernel_version`, etc. Uma afirmação simples como
  `assert!(!metrics.system_info.hostname.is_empty())` já cobriria o campo.

- **[SUGESTÃO] `update_metrics` não tem teste**

  Por ser `async` e depender de tempo real, é difícil testar diretamente.
  Mas o comportamento de delta (entradas zeradas na primeira chamada) poderia
  ser verificado com um mock ou injeção de função.

---

## QML — Estilo e Convenções

### Tamanho de arquivos

- **[AVISO] `SensorsTab.qml` — 272 linhas**
- **[AVISO] `SystemTab.qml` — 270 linhas**
- **[AVISO] `CpuTab.qml` — 254 linhas**

  O limite do AGENTS.md é 700 linhas para arquivos, então não há violação formal.
  Porém os três arquivos têm potencial de crescer; dividir seções repetitivas
  em subcomponentes seria preventivo.

### Duplicação de funções utilitárias

- **[AVISO] `fmtUptime` definida em 3 arquivos — `FullRepresentation.qml`, `CpuTab.qml`, `SystemTab.qml`**

  As três implementações são semanticamente equivalentes (mesma lógica de
  dias/horas/minutos), mas com formatação interna ligeiramente diferente
  (ternário vs if-block). Qualquer correção futura precisa ser replicada.

  Sugestão: mover para `Theme.qml` como função utilitária ou criar
  `components/Utils.qml` com funções compartilhadas.

- **[AVISO] `fmtBytes` duplicada em `NetworkTab.qml` e `SystemTab.qml`**

  Implementações idênticas.

- **[AVISO] `fmtOne` duplicada em `MemoryTab.qml`, `DiskTab.qml` e `SystemTab.qml`**

  Implementações idênticas (`Number(value).toFixed(1)`).

- **[AVISO] `fmtRate` duplicada em `DiskTab.qml` e `NetworkTab.qml`**

  Implementações idênticas.

### Cores hardcoded fora do Theme

- **[AVISO] `#a78bfa` (roxo do swap) em 3 arquivos**

  `MemoryTab.qml:62`, `MemoryTab.qml:105`, `SystemTab.qml:195` usam `"#a78bfa"`
  diretamente. Se a paleta mudar, são 3 locais para atualizar.

  Sugestão: adicionar `readonly property color swapColor: "#a78bfa"` ao `Theme.qml`.

- **[AVISO] `Qt.rgba(…)` hardcoded em 6 locais nas tabs**

  As cores de preenchimento dos `HistoryChart` são derivações com alpha de cores
  do tema, mas estão literais nas tabs em vez de calculadas a partir do
  `theme.cpuColor` etc.

  Sugestão: `HistoryChart` poderia calcular `fillColor` automaticamente a partir
  de `strokeColor` com alpha fixo, eliminando os 6 literais.

### Nomes de variáveis

- **[SUGESTÃO] `var n`, `var t` em closures de temperatura**

  `CpuTab.qml:38,55` usa `var n` para o nome do chip e
  `SensorsTab.qml:195` / `SystemTab.qml:252` usam `var t` para temperatura.
  Nomes de uma letra dificultam leitura em closures aninhados.

  Sugestão: `var chipName`, `var tempCelsius`.

### Aninhamento QML

- **[AVISO] `SensorsTab.qml:195` — lógica de cor em nível de indentação ≈ 7**

  O bloco `accentColor: { var t = ...; if (t >= 85) ... }` dentro de um
  `delegate` dentro de um `Repeater` dentro de outro `Repeater` dentro de
  `MetricCard` atinge ~7 níveis de indentação.

  Sugestão: extrair para função `temperatureAccentColor(celsius)` no `root`.

---

## Segurança

Nenhum problema encontrado.

---

## Manutenção

- **[SUGESTÃO] `collector.rs` — `get_disk_metrics` e `get_sensor_metrics` poderiam
  usar funções auxiliares privadas**

  `get_disk_metrics` tem um closure de ~20 linhas dentro de `.map()`.
  `get_sensor_metrics` tem um bloco `else` com `filter_map` de ~15 linhas.
  Extrair para `fn build_disk_info(…)` e `fn sysinfo_temperatures(…)` melhoraria
  a legibilidade sem alterar comportamento.

---

## Resumo

| Categoria | Erros | Avisos | Sugestões |
|---|---|---|---|
| Rust — Aninhamento | — | 2 | — |
| Rust — Comentários | — | 1 | — |
| Rust — Testes | — | 3 | 1 |
| QML — Duplicação | — | 4 | — |
| QML — Cores hardcoded | — | 2 | — |
| QML — Nomes | — | — | 1 |
| QML — Aninhamento | — | 1 | — |
| Manutenção | — | — | 1 |
| **Total** | **0** | **13** | **3** |

**Próximos passos sugeridos (por impacto):**
1. Centralizar `fmtUptime`, `fmtBytes`, `fmtOne`, `fmtRate` em `Theme.qml`
2. Adicionar `swapColor` ao `Theme.qml` e eliminar os 3 literais `#a78bfa`
3. Adicionar testes para `compute_cpu_percents`, `device_basename` e `temp*_input` no hwmon
4. Documentar `hwmon.rs` com comentários em PT-BR
5. Extrair `compute_disk_rates` para reduzir o aninhamento no collector

---
_Relatório salvo em: `.pi/audit/2026-04-21-quality-repositorio-completo.md`_
