# Plano: mover histórico temporal para o backend em todas as tabs

**Data:** 2026-04-23
**Autor:** agente-plan
**Status:** aprovado

---

## Objetivo

Fazer com que os históricos temporais deixem de depender da UI QML aberta e passem a ser mantidos no backend Rust/DBus em memória, permitindo monitoramento contínuo em background enquanto o serviço estiver ativo.

O plano cobre:
- migrar os históricos já existentes (CPU, RAM, GPU, Disk, Network) para o backend;
- definir e expor históricos também para **Sensors** e **System**, para que todas as tabs tenham base histórica no backend.

## Escopo

**Dentro do escopo:**
- criar estrutura de histórico circular no backend Rust;
- alimentar esse histórico a partir dos ciclos já existentes de coleta rápida/lenta;
- expor um novo contrato DBus para consultar histórico;
- adaptar o frontend QML para consumir histórico vindo do backend;
- remover a dependência de `root.expanded` para o acúmulo histórico;
- planejar histórico para **todas as tabs**: CPU, RAM, GPU, Disk, Network, Sensors e System.

**Fora do escopo:**
- persistência em disco entre reinícios do serviço;
- alterações visuais grandes nos componentes QML;
- redesenho completo do contrato DBus existente além do necessário;
- histórico por processo individual, por disco individual ou por interface individual como primeira etapa.

---

## Alternativas e Trade-offs

### Alternativa A — histórico no backend, mas só para as tabs que já têm gráfico
**Prós**
- menor esforço e menor risco;
- resolve imediatamente o problema principal.

**Contras**
- “todas as tabs” ficaria incompleto;
- Sensors e System continuariam sem base histórica padronizada.

### Alternativa B — histórico no backend para todas as tabs, em duas fases
**Prós**
- resolve o bug arquitetural e deixa o projeto consistente;
- cria uma API de histórico única e extensível;
- permite adicionar gráficos novos em Sensors/System sem retrabalho no backend.

**Contras**
- mudança maior no contrato de dados;
- exige mais cuidado com compatibilidade e tamanho de payload.

### Recomendação
Adotar a **Alternativa B**, em duas fases:

1. **Fase 1**: o backend passa a manter e expor os históricos já usados pela UI.
2. **Fase 2**: o backend também passa a manter históricos de Sensors/System, mesmo que a UI inicialmente só consuma parte deles.

---

## Arquivos Afetados

| Arquivo | Ação | Motivo |
|---|---|---|
| `src/monitor/models.rs` | modificar | adicionar tipos serializáveis para séries históricas e payload agregado de histórico |
| `src/monitor/collector.rs` | modificar | incorporar armazenamento histórico ao `SystemMonitor` e alimentar buffers nos ciclos rápido/lento |
| `src/monitor/mod.rs` | modificar | reexportar novos tipos de histórico |
| `src/lib.rs` | modificar | expor helpers para coletar/serializar histórico |
| `src/dbus.rs` | modificar | adicionar método DBus para retornar histórico em JSON |
| `plasma/contents/ui/main.qml` | modificar | remover construção local do histórico como fonte primária e consumir histórico do backend |
| `plasma/contents/ui/FullRepresentation.qml` | modificar | receber payload histórico vindo do backend |
| `plasma/contents/ui/tabs/CpuTab.qml` | ajustar | continuar consumindo série, mas agora originada no backend |
| `plasma/contents/ui/tabs/MemoryTab.qml` | ajustar | idem |
| `plasma/contents/ui/tabs/GpuTab.qml` | ajustar | idem |
| `plasma/contents/ui/tabs/DiskTab.qml` | ajustar | idem |
| `plasma/contents/ui/tabs/NetworkTab.qml` | ajustar | idem |
| `plasma/contents/ui/tabs/SensorsTab.qml` | modificar | preparar consumo de histórico de sensores |
| `plasma/contents/ui/tabs/SystemTab.qml` | modificar | preparar consumo de histórico de sistema |
| `docs/frontend.md` | modificar | documentar nova fonte do histórico |
| `docs/backend.md` | modificar | documentar novo endpoint/contrato DBus |
| `docs/models.md` | modificar | documentar novos modelos de histórico |

---

## Modelo de Dados Proposto

### 1. Estrutura base de série
Criar um tipo serializável no backend para substituir o buffer circular do QML como fonte da verdade, algo na linha de:

- `HistorySeries<T>`
  - `buffer`
  - `start`
  - `count`
  - `capacity`
  - opcionalmente `sample_interval_ms`

Como o frontend já entende `{ buffer, start, count }`, vale manter compatibilidade estrutural para reduzir retrabalho no QML.

### 2. Payload agregado por tab
Criar um payload agregado, por exemplo `HistoryMetrics`, contendo:

- `cpu_usage`
- `memory_usage`
- `gpu_usage`
- `disk_read`
- `disk_write`
- `network_download`
- `network_upload`

E também novos campos para completar todas as tabs:

- `sensor_average_temperature`
- `sensor_hottest_temperature`
- `sensor_hottest_cpu_temperature`
- `sensor_hottest_gpu_temperature`
- `sensor_total_fan_rpm` ou `sensor_highest_fan_rpm`
- `sensor_total_power_watts` ou `sensor_highest_power_watts`

- `system_load_1`
- `system_load_5`
- `system_load_15`
- `system_process_count`

### 3. Metadados do histórico
No mesmo payload:
- `history_duration_ms`
- `fast_sample_interval_ms`
- `slow_sample_interval_ms`

Isso evita que a UI precise inferir tudo localmente.

---

## Sequência de Execução

### 1. Definir contrato histórico no backend
**Arquivos:** `src/monitor/models.rs`, `src/monitor/mod.rs`
**O que fazer:**
- adicionar tipos de série histórica e payload agregado de histórico;
- garantir serialização com nomes estáveis em snake_case;
- manter formato compatível com `HistoryChart.qml`.
**Dependências:** nenhuma

### 2. Incorporar armazenamento histórico ao `SystemMonitor`
**Arquivos:** `src/monitor/collector.rs`
**O que fazer:**
- adicionar buffers circulares ao estado do `SystemMonitor`;
- inicializar capacidades com base na janela desejada de retenção;
- centralizar helper de append/clamp no Rust.
**Dependências:** passo 1

### 3. Alimentar histórico no ciclo rápido
**Arquivos:** `src/monitor/collector.rs`
**O que fazer:**
Após `update_fast_metrics()`, registrar:
- uso de CPU;
- uso de RAM;
- throughput agregado de rede;
- throughput agregado de disco;
- load average relevante para System;
- contagem de processos quando fizer sentido.

**Justificativa:** essas métricas já são calculadas no caminho quente e não devem depender da UI.
**Dependências:** passo 2

### 4. Alimentar histórico no ciclo lento
**Arquivos:** `src/monitor/collector.rs`, possivelmente `src/monitor/gpu.rs`
**O que fazer:**
Após `refresh_slow_metrics()`, registrar:
- uso da GPU principal;
- temperatura média e pico de sensores;
- hottest CPU/GPU;
- agregados elétricos/fans para a tab Sensors.
**Dependências:** passos 2 e 3

### 5. Expor histórico via DBus
**Arquivos:** `src/lib.rs`, `src/dbus.rs`
**O que fazer:**
- adicionar helper `collect_history_metrics_json(...)`;
- criar método DBus dedicado, por exemplo `HistoryMetricsJson`;
- decidir se ele retorna apenas histórico ou histórico + metadados.

**Recomendação:** endpoint separado, para não inflar `FastMetricsJson` nem `SlowMetricsJson`.
**Dependências:** passos 1–4

### 6. Adaptar `main.qml` para usar histórico do backend
**Arquivos:** `plasma/contents/ui/main.qml`
**O que fazer:**
- remover `createHistorySeries()` e `appendHistory()` como fonte principal;
- trocar o preenchimento local por atualização a partir do payload DBus de histórico;
- manter, se necessário, um fallback local temporário só para compatibilidade com backends antigos;
- desacoplar o histórico de `root.expanded`.
**Dependências:** passo 5

### 7. Conectar o histórico às tabs existentes
**Arquivos:** `plasma/contents/ui/FullRepresentation.qml`, `plasma/contents/ui/tabs/CpuTab.qml`, `plasma/contents/ui/tabs/MemoryTab.qml`, `plasma/contents/ui/tabs/GpuTab.qml`, `plasma/contents/ui/tabs/DiskTab.qml`, `plasma/contents/ui/tabs/NetworkTab.qml`
**O que fazer:**
- manter a API visual dos tabs o mais estável possível;
- trocar apenas a origem das séries;
- validar máximos dinâmicos de Disk/Network com série vinda do backend.
**Dependências:** passo 6

### 8. Estender Sensors e System para usar histórico do backend
**Arquivos:** `plasma/contents/ui/tabs/SensorsTab.qml`, `plasma/contents/ui/tabs/SystemTab.qml`, `plasma/contents/ui/FullRepresentation.qml`
**O que fazer:**
- adicionar props de histórico para essas tabs;
- decidir quais séries serão exibidas primeiro:
  - **Sensors**: média, pico, hottest CPU, hottest GPU;
  - **System**: load 1m, processos.
- se não houver gráfico novo imediato, ao menos deixar a passagem de props e contrato pronta.
**Dependências:** passos 5–7

### 9. Atualizar documentação
**Arquivos:** `docs/frontend.md`, `docs/backend.md`, `docs/models.md`
**O que fazer:**
- remover a afirmação de que histórico só acumula com popup expandido;
- documentar o novo endpoint e os novos modelos;
- registrar claramente que o backend é a fonte da verdade do histórico em memória.
**Dependências:** passos 1–8

### 10. Validar comportamento e regressões
**Arquivos:** backend e frontend afetados
**O que fazer:**
- validar que histórico cresce com UI fechada/compacta;
- validar que abrir a UI depois mostra séries já acumuladas;
- validar que `FastMetricsJson` continua leve;
- rodar build/test/lint do projeto.
**Dependências:** todos os anteriores

---

## Riscos e Mitigações

| Risco | Probabilidade | Mitigação |
|---|---|---|
| Payload DBus de histórico ficar grande demais | média | endpoint separado, séries agregadas e janela fixa |
| Divergência entre intervalos rápido/lento e leitura do gráfico | média | expor `sample_interval_ms` e documentar séries rápidas vs lentas |
| GPU/Sensors produzirem amostras esparsas | alta | aceitar taxa de amostragem diferente por série e deixar isso explícito no modelo |
| Regressão de compatibilidade com UI atual | média | manter formato `{ buffer, start, count }` |
| A UI continuar duplicando estado histórico | média | remover append local como fonte primária |
| Complexidade excessiva em Sensors/System | média | fasear: backend completo primeiro, visual incremental depois |
| Uso de memória aumentar | baixa | buffers fixos/circulares e retenção de apenas ~5 min |

---

## Critérios de Conclusão

- [ ] O backend mantém histórico contínuo sem depender da UI expandida
- [ ] CPU, RAM, GPU, Disk e Network passam a consumir histórico originado no Rust
- [ ] Existe contrato de histórico para Sensors e System no backend
- [ ] A UI consegue abrir depois de um período fechada e exibir histórico já acumulado
- [ ] `FastMetricsJson` e `SlowMetricsJson` continuam funcionais e sem inflar indevidamente
- [ ] Documentação atualizada para refletir a nova arquitetura
- [ ] `make build` passa
- [ ] `make test` passa
- [ ] `make lint` passa
