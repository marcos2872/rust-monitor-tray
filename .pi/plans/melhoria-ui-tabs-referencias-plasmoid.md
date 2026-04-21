# Plano: Melhoria visual das tabs do plasmoid com base nas referências

**Data:** 2026-04-20
**Autor:** agente-plan
**Status:** aprovado

---

## Objetivo

Refatorar o popup tabulado do plasmoid para aproximar a UI visual das referências em `referencias/`, especialmente nas tabs `CPU`, `RAM`, `Disk`, `Network` e `Sensors`, mantendo a arquitetura atual em QML e sem regressões no backend DBus/hwmon. O foco é melhorar hierarquia visual, blocos principais, gráficos, densidade e legibilidade para deixar o popup mais próximo dos mockups de referência.

## Escopo

**Dentro do escopo:**
- refatorar o layout visual das tabs `CPU`, `RAM`, `Disk`, `Network` e `Sensors`
- criar componentes visuais reutilizáveis para métricas hero, gauges/rings simples, painéis de resumo e listas densas
- reposicionar e destacar os gráficos históricos de 5 minutos
- melhorar hierarquia visual, espaçamento, tipografia, superfícies e agrupamento de informações
- adaptar a aba `Disk` para destacar um disco principal e secundários
- adaptar a aba `Network` para números grandes de RX/TX e histórico como bloco principal
- adaptar a aba `Sensors` para seções mais próximas da referência visual
- revisar o container do popup para acomodar os novos layouts sem scroll desnecessário

**Fora do escopo:**
- mudar a arquitetura DBus ou a forma de coleta de métricas do backend
- adicionar compatibilidade com outros ambientes além de KDE/Plasma
- implementar novas fontes de dados só para atingir fidelidade visual
- recriar as referências de forma pixel-perfect
- mexer no widget compacto do painel, exceto por impacto colateral inevitável

---

## Arquivos Afetados

| Arquivo | Ação | Motivo |
|---|---|---|
| `plasma/contents/ui/FullRepresentation.qml` | modificar | ajustar tamanho/comportamento do popup e a montagem das tabs |
| `plasma/contents/ui/Theme.qml` | modificar | expandir tokens visuais para aproximar o visual das referências |
| `plasma/contents/ui/components/MetricCard.qml` | modificar | permitir cards com estilos mais próximos de painéis hero |
| `plasma/contents/ui/components/SectionHeader.qml` | modificar | melhorar hierarquia de títulos e subtítulos |
| `plasma/contents/ui/components/MetricRow.qml` | modificar | ajustar linhas de detalhe com densidade e alinhamento mais estáveis |
| `plasma/contents/ui/components/MetricBar.qml` | modificar | adaptar barras ao novo visual |
| `plasma/contents/ui/components/HistoryChart.qml` | modificar | destacar e aproximar os históricos das referências |
| `plasma/contents/ui/components/HeroMetric.qml` | criar | exibir métricas principais com valor grande e unidade |
| `plasma/contents/ui/components/RingGauge.qml` | criar | compor indicadores circulares inspirados nas referências |
| `plasma/contents/ui/components/SensorValueList.qml` | criar | padronizar listas visuais de sensores e leituras elétricas |
| `plasma/contents/ui/components/FanRow.qml` | criar | padronizar linhas de fan com RPM e barra/duty |
| `plasma/contents/ui/tabs/CpuTab.qml` | modificar | reorganizar em hero metrics + histórico + detalhes |
| `plasma/contents/ui/tabs/MemoryTab.qml` | modificar | reorganizar em hero metrics + histórico + detalhes |
| `plasma/contents/ui/tabs/DiskTab.qml` | modificar | destacar disco principal e reduzir aparência de lista simples |
| `plasma/contents/ui/tabs/NetworkTab.qml` | modificar | destacar taxas atuais e histórico como foco principal |
| `plasma/contents/ui/tabs/SensorsTab.qml` | modificar | aproximar seções `Fans`, `Temperature`, `Voltage`, `Current`, `Power` da referência |
| `referencias/*.webp` | consultar | guiar a implementação e a validação visual |

---

## Alternativas e Trade-offs

### Alternativa A — Refatoração incremental sobre os componentes atuais
**Prós:**
- menor risco de quebrar popup e tabs
- reaproveita componentes já existentes (`MetricCard`, `HistoryChart`, `MetricRow`)
- implementação mais rápida e validável por etapas
- reduz retrabalho estrutural

**Contras:**
- limita parte da fidelidade visual
- alguns componentes atuais carregam um visual mais técnico do que o desejado

### Alternativa B — Novo kit visual dedicado para as tabs
**Prós:**
- maior liberdade para aproximar a UI das referências
- separa melhor a UI nova da base atual

**Contras:**
- maior custo de implementação e manutenção
- mais risco de inconsistência visual e regressões
- maior retrabalho em componentes próximos dos já existentes

### Recomendação
Adotar a **Alternativa A**, com criação pontual de novos componentes visuais reutilizáveis. Essa abordagem oferece o melhor equilíbrio entre fidelidade visual, risco técnico e velocidade de execução.

---

## Sequência de Execução

### 1. Consolidar tokens visuais
**Arquivos:** `plasma/contents/ui/Theme.qml`, `plasma/contents/ui/components/MetricCard.qml`, `plasma/contents/ui/components/SectionHeader.qml`

**O que fazer:**
- ampliar o tema com tamanhos, opacidades, superfícies, contrastes e espaçamentos
- preparar variantes de card para blocos hero e listas densas
- ajustar títulos e subtítulos para melhor hierarquia visual

**Dependências:** nenhuma

### 2. Criar componentes base para o novo visual
**Arquivos:**
- `plasma/contents/ui/components/HeroMetric.qml`
- `plasma/contents/ui/components/RingGauge.qml`
- `plasma/contents/ui/components/FanRow.qml`
- `plasma/contents/ui/components/SensorValueList.qml`
- `plasma/contents/ui/components/HistoryChart.qml`

**O que fazer:**
- criar primitives visuais para métricas principais com números grandes
- criar um gauge circular simples em QML para aproximar CPU/RAM das referências
- padronizar exibição de fans e listas de sensores
- evoluir o gráfico histórico já existente para ganhar mais destaque visual

**Dependências:** passo 1

### 3. Refatorar a aba CPU
**Arquivos:** `plasma/contents/ui/tabs/CpuTab.qml`

**O que fazer:**
- destacar as métricas principais em blocos hero/ring
- manter o histórico de 5 minutos como elemento central da aba
- reorganizar detalhes por núcleo e frequência abaixo do bloco principal
- reduzir aparência de formulário/lista técnica

**Dependências:** passos 1 e 2

### 4. Refatorar a aba RAM
**Arquivos:** `plasma/contents/ui/tabs/MemoryTab.qml`

**O que fazer:**
- destacar uso atual como hero metric ou ring principal
- manter o histórico de 5 minutos com protagonismo
- resumir detalhes em poucos blocos (`usada`, `livre`, `swap`)
- aproximar a organização da referência `ram.webp`

**Dependências:** passos 1 e 2

### 5. Refatorar a aba Disk
**Arquivos:** `plasma/contents/ui/tabs/DiskTab.qml`

**O que fazer:**
- escolher um disco principal por heurística (por exemplo `/` ou maior uso)
- mostrar nome, barra principal, usado/total/% em destaque
- renderizar discos secundários abaixo em formato reduzido
- aproximar a aba da ideia de um painel principal em vez de lista crua

**Dependências:** passos 1 e 2

### 6. Refatorar a aba Network
**Arquivos:** `plasma/contents/ui/tabs/NetworkTab.qml`

**O que fazer:**
- destacar download e upload atual em números grandes
- usar o histórico como área principal da aba
- manter interfaces e totais como detalhes secundários abaixo
- aproximar o layout da referência `network.webp`

**Dependências:** passos 1 e 2

### 7. Refatorar a aba Sensors
**Arquivos:** `plasma/contents/ui/tabs/SensorsTab.qml`

**O que fazer:**
- reorganizar em blocos visuais mais próximos da referência `sensors.webp`
- manter seções separadas para `Fans`, `Temperature`, `Voltage`, `Current` e `Power`
- melhorar o visual das linhas de fan e das listas de sensores
- manter compatibilidade com cobertura parcial de sensores dependendo do hardware

**Dependências:** passos 1 e 2

### 8. Ajustar o container do popup
**Arquivos:** `plasma/contents/ui/FullRepresentation.qml`

**O que fazer:**
- revisar largura/altura do popup
- garantir que tabs curtas não mostrem scroll desnecessário
- acomodar o novo layout sem quebrar a navegação por tabs

**Dependências:** passos 3 a 7

### 9. Validação final
**Arquivos:** todos os QML afetados

**O que fazer:**
- validar sintaxe QML
- executar `cargo test`
- executar `cargo build`
- executar `make kde-refresh`
- comparar cada tab com sua referência correspondente e ajustar detalhes finos

**Dependências:** passos 1 a 8

---

## Riscos e Mitigações

| Risco | Probabilidade | Mitigação |
|---|---|---|
| O popup voltar a apresentar scroll em tabs curtas | média | validar altura efetiva de cada aba e ajustar densidade vertical |
| Os novos componentes visuais aumentarem a complexidade do QML | média | manter componentes pequenos, focados e reutilizáveis |
| Gauges circulares ficarem pesados ou inconsistentes | média | implementar versão simples e prever fallback para métricas hero sem gauge |
| A aba Disk não corresponder à referência por falta de histórico de IO | alta | aproximar o layout visual sem prometer equivalência total de dados |
| A aba Sensors continuar distante da referência por falta de energy | média | assumir aproximação visual com os dados reais já disponíveis |
| Perda de legibilidade no tema escuro do Plasma | média | validar contraste e refiná-lo no `Theme.qml` |

---

## Critérios de Conclusão

- [ ] `cargo test` passa sem erros
- [ ] `cargo build` passa sem erros
- [ ] `make kde-refresh` recarrega o plasmoid sem quebrar o popup
- [ ] Cada tab tem layout claramente inspirado na respectiva referência
- [ ] CPU, RAM e Network mantêm histórico de 5 minutos em posição visual destacada
- [ ] Disk deixa de parecer apenas uma lista e passa a destacar um disco principal
- [ ] Sensors fica organizada em seções visuais próximas da referência
- [ ] O popup não apresenta scroll desnecessário em tabs curtas
- [ ] O visual permanece legível e coerente no tema atual do Plasma

---

## Próximo passo sugerido

O plano está pronto. Para executar, saia do modo de planejamento e use o modo normal ou a skill correspondente de implementação.
