# Componentes QML — Monitor Tray

Componentes reutilizáveis em `plasma/contents/ui/components/`. Todos importam `Theme.qml` via `".."`.

---

## MetricCard

**Arquivo:** `MetricCard.qml`  
**Base:** `Item`

Container padrão para agrupar métricas. Aceita conteúdo via `default property alias cardContent`.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `title` | `string` | `""` | Título da seção |
| `subtitle` | `string` | `""` | Subtítulo ou descrição |
| `hero` | `bool` | `false` | Estilo hero (fundo elevado, bordas maiores) |
| `contentSpacing` | `int` | `theme.spacingS` | Espaçamento entre itens filhos |

---

## HeroMetric

**Arquivo:** `HeroMetric.qml`  
**Base:** `Item` com `implicitHeight: 110`

Card de destaque com valor grande, unidade, rótulo e nota de rodapé.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `label` | `string` | `""` | Rótulo superior |
| `value` | `string` | `"0"` | Valor principal (grande) |
| `unit` | `string` | `""` | Unidade ao lado do valor |
| `accentColor` | `color` | `#60a5fa` | Cor do valor e da borda |
| `footnote` | `string` | `""` | Texto de rodapé (menor) |

---

## RingGauge

**Arquivo:** `RingGauge.qml`  
**Base:** `Item`

Gauge circular com arco colorido, texto central e rótulo.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `value` | `real` | `0` | Valor atual |
| `maximumValue` | `real` | `100` | Valor máximo para cálculo do arco |
| `centerText` | `string` | `"0%"` | Texto exibido no centro |
| `label` | `string` | `""` | Rótulo abaixo do texto central |
| `footnote` | `string` | `""` | Texto abaixo do gauge |
| `accentColor` | `color` | `#60a5fa` | Cor do arco ativo |
| `gaugeSize` | `int` | `112` | Diâmetro do gauge em px |

---

## HistoryChart

**Arquivo:** `HistoryChart.qml`  
**Base:** `Item` com `implicitHeight: theme.chartHeight + 30`

Gráfico de área com linha de contorno. `fillColor` é derivado automaticamente de `strokeColor` com alpha 0.18.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `values` | `var` (array) | `[]` | Array de valores numéricos |
| `strokeColor` | `color` | `#60a5fa` | Cor da linha |
| `fillColor` | `color` | `Qt.rgba(r,g,b,0.18)` | Cor do preenchimento (auto-derivado) |
| `minimumValue` | `real` | `0` | Mínimo do eixo Y |
| `maximumValue` | `real` | `-1` | Máximo fixo (`-1` = auto) |
| `maxLabel` | `string` | `""` | Label do topo direito |
| `minLabel` | `string` | `"0"` | Label do rodapé direito |
| `leftFooterText` | `string` | `"5 min atrás"` | Label do rodapé esquerdo |
| `rightFooterText` | `string` | `"Agora"` | Label do rodapé central-direito |

**Layout dos labels:**
```
                              maxLabel
[gráfico]
leftFooterText ─── rightFooterText  minLabel
```

---

## MetricRow

**Arquivo:** `MetricRow.qml`  
**Base:** `RowLayout`

Linha com ponto colorido, rótulo e valor alinhados.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `label` | `string` | `""` | Texto à esquerda |
| `value` | `string` | `""` | Texto à direita (bold) |
| `accentColor` | `color` | `transparent` | Cor do ponto indicador (oculto se transparent) |
| `dense` | `bool` | `false` | Fonte menor e espaçamento reduzido |

---

## MetricBar

**Arquivo:** `MetricBar.qml`  
**Base:** `Item`

Barra de progresso horizontal com label e percentual.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `label` | `string` | `""` | Texto à esquerda |
| `value` | `real` | `0` | Valor de 0 a 100 |
| `barColor` | `color` | `#60a5fa` | Cor do preenchimento |
| `barHeight` | `int` | `12` | Altura da barra em px |
| `suffix` | `string` | `"%"` | Sufixo do valor exibido à direita |

---

## SectionHeader

**Arquivo:** `SectionHeader.qml`  
**Base:** `RowLayout`

Cabeçalho de seção com título e subtítulo opcionais.

| Prop | Tipo | Descrição |
|---|---|---|
| `title` | `string` | Texto principal |
| `subtitle` | `string` | Texto secundário (menor, cor reduzida) |

---

## StatusChip

**Arquivo:** `StatusChip.qml`  
**Base:** `Rectangle` com `radius: height / 2`

Chip de status pill com cor de fundo semitransparente.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `text` | `string` | `""` | Texto exibido |
| `chipColor` | `color` | `#64748b` | Cor do texto e fundo (com alpha 0.18) |

---

## FanRow

**Arquivo:** `FanRow.qml`  
**Base:** `ColumnLayout`

Linha de ventilador com RPM, barra de duty cycle e percentual.

| Prop | Tipo | Descrição |
|---|---|---|
| `label` | `string` | Nome do ventilador |
| `rpm` | `real` | Rotações por minuto |
| `dutyPercent` | `real` | Duty cycle em % (`-1` = oculto) |
| `accentColor` | `color` | Cor da barra de duty |

---

## SensorValueList

**Arquivo:** `SensorValueList.qml`  
**Base:** `ColumnLayout`

Lista genérica de sensores com `label` e valor formatado. Usado por Voltage, Current e Power no SensorsTab.

| Prop | Tipo | Padrão | Descrição |
|---|---|---|---|
| `items` | `var` (array) | `[]` | Array de objetos com `label` + `valueProp` |
| `valueProp` | `string` | `"value"` | Nome da propriedade de valor no objeto |
| `decimals` | `int` | `1` | Casas decimais |
| `suffix` | `string` | `""` | Sufixo (ex.: `" V"`, `" A"`, `" W"`) |
| `accentColor` | `color` | `transparent` | Cor do ponto em cada linha |
| `emptyText` | `string` | `"Sem dados"` | Texto quando `items` está vazio |
