# Plano: UI do Plasmoid KDE para o monitor de sistema

**Data:** 2026-04-20
**Autor:** agente-plan
**Status:** aprovado

---

## Objetivo

Definir a UI do frontend KDE/Plasma como um Plasmoid com visual nativo, legibilidade alta e foco em acesso rápido às métricas de CPU, memória, disco, rede e sistema. O plano descreve a estrutura visual, componentes, estados, comportamento do popup e os contratos mínimos que a UI exigirá do backend Rust.

## Escopo

**Dentro do escopo:**
- layout visual do item no painel
- layout do popup principal
- seções, cards e hierarquia de informação
- estados visuais (loading, erro, sem dados)
- comportamento de abertura/fechamento
- decisões de responsividade e densidade de informação
- mapeamento entre métricas do backend e componentes da UI
- convenções visuais alinhadas ao KDE/Breeze

**Fora do escopo:**
- implementação do backend DBus
- tema custom estilo Apple
- configurações avançadas do usuário
- gráficos históricos longos
- GPU
- notificações e alertas inteligentes

---

## Arquivos Afetados

| Arquivo | Ação | Motivo |
|---|---|---|
| `plasma/metadata.json` | criar | registrar o Plasmoid e suas capacidades |
| `plasma/contents/ui/main.qml` | criar | ponto de entrada do widget no painel |
| `plasma/contents/ui/CompactRepresentation.qml` | criar | representação compacta no painel |
| `plasma/contents/ui/FullRepresentation.qml` | criar | popup expandido principal |
| `plasma/contents/ui/components/MetricCard.qml` | criar | card reutilizável por seção |
| `plasma/contents/ui/components/MetricBar.qml` | criar | barra visual reutilizável |
| `plasma/contents/ui/components/MetricRow.qml` | criar | linha label/valor consistente |
| `plasma/contents/ui/components/SectionHeader.qml` | criar | cabeçalho padronizado das seções |
| `plasma/contents/ui/components/StatusChip.qml` | criar | selo compacto para estado/cor |
| `plasma/contents/ui/theme.js` ou `Theme.qml` | criar | tokens simples de spacing e cores sem quebrar o tema KDE |
| `README.md` | modificar | documentar o comportamento visual esperado |

---

## Princípios de UI

### 1. Aparência nativa do KDE
- Usar Kirigami e Plasma Components em vez de desenhar tudo manualmente.
- Herdar tipografia, padding, cores e contraste do tema Breeze/Breeze Dark.
- Evitar visual “custom demais” que entre em conflito com o Plasma.

### 2. Leitura rápida em 2 níveis
- **Nível 1:** item do painel mostra um resumo mínimo.
- **Nível 2:** popup mostra detalhes organizados em cards.

### 3. Pouca densidade por bloco
- Cada card deve mostrar poucas informações-chave.
- Detalhes extensos ficam em listas secundárias dentro do próprio card.

### 4. Consistência visual
- Mesmo padrão de barras, labels e alinhamento em todas as seções.
- Mesma lógica de cor para severidade: baixa / média / alta.

---

## Estrutura da UI

## 1. Representação compacta no painel

### Objetivo
Ser legível em tamanhos pequenos de painel sem depender de texto longo.

### Proposta
- Ícone compacto com duas barras horizontais:
  - barra superior = CPU
  - barra inferior = RAM
- Tooltip curto opcional:
  - `CPU 42% • RAM 46%`

### Regras
- Não usar texto detalhado no painel.
- A cor da barra deve representar severidade da métrica.
- Se faltar espaço, a forma do ícone ainda precisa ser compreensível.

---

## 2. Popup principal

### Dimensões iniciais sugeridas
- largura: `360–420 px`
- altura: automática com scroll a partir de certo limite
- bordas e cantos herdados de Kirigami/Plasma

### Estrutura vertical
1. Cabeçalho
2. Cards principais
3. Rodapé curto com ações/configuração futura

### Cabeçalho
Conteúdo sugerido:
- título: `Monitor do Sistema`
- subtítulo curto: hostname ou uptime
- timestamp da última atualização opcional

### Corpo
Cards em ordem:
1. CPU
2. Memória
3. Disco
4. Rede
5. Sistema

### Rodapé
Inicialmente simples:
- texto de atualização (`Atualizado agora` / `há 1s`)
- ação futura `Configurações` (desabilitada ou omitida até existir)

---

## 3. Card de CPU

### Informações principais
- barra total de uso
- percentual total
- frequência atual
- quantidade de núcleos

### Informações secundárias
- lista por núcleo em grade de 2 colunas quando houver muitos núcleos
- cada núcleo com mini barra e percentual

### Layout sugerido
- cabeçalho do card: `CPU`
- linha principal: barra grande + valor total
- grid inferior: `Freq`, `Núcleos`
- bloco secundário: `Uso por núcleo`

### Regras visuais
- barra principal mais larga que as demais
- núcleos em mini barras compactas
- se houver muitos núcleos, mostrar top N e permitir expansão futura

---

## 4. Card de Memória

### Informações principais
- barra de RAM
- percentual de uso
- usado / total

### Informações secundárias
- memória disponível
- swap usada / total
- barra de swap apenas se houver swap configurada

### Layout sugerido
- cabeçalho: `Memória`
- barra principal de RAM
- grid com:
  - `Usada`
  - `Livre`
  - `Swap`

### Regras visuais
- RAM e swap devem ser visualmente distintas
- swap não deve competir visualmente com RAM

---

## 5. Card de Disco

### Informações principais
- barra do uso agregado
- usado / total
- livre

### Informações secundárias
- lista por partição
- cada partição com mount point, barra curta e percentual

### Layout sugerido
- cabeçalho: `Disco`
- barra agregada
- lista de partições abaixo

### Regras visuais
- priorizar partições relevantes (`/`, `/home`) primeiro
- nomes muito longos devem ser truncados com elipse

---

## 6. Card de Rede

### Informações principais
- total RX
- total TX
- número de interfaces

### Informações secundárias
- lista por interface
- cada interface com:
  - nome
  - RX total
  - TX total

### Evolução futura opcional
- taxa instantânea RX/s e TX/s

### Layout sugerido
- cabeçalho: `Rede`
- duas métricas principais lado a lado (`RX`, `TX`)
- lista de interfaces abaixo

### Regras visuais
- usar setas ou chips discretos para RX/TX
- evitar linhas muito longas; quebrar em duas linhas se necessário

---

## 7. Card de Sistema

### Informações principais
- uptime
- load average

### Informações secundárias possíveis
- hostname
- kernel
- versão do sistema (fase posterior)

### Layout sugerido
- cabeçalho: `Sistema`
- duas ou três linhas simples de status

---

## Componentes reutilizáveis

### `MetricCard.qml`
Responsável por:
- container visual uniforme
- título
- área de conteúdo
- espaçamento padrão

### `MetricBar.qml`
Responsável por:
- barra de progresso temática
- cores por severidade
- label percentual opcional

### `MetricRow.qml`
Responsável por:
- linha label/valor consistente
- truncamento seguro
- alinhamento correto

### `SectionHeader.qml`
Responsável por:
- título + subtítulo opcional
- separação sem excesso de ruído visual

### `StatusChip.qml`
Responsável por:
- pequenos selos para estados como `OK`, `Médio`, `Alto`
- uso opcional em CPU/RAM/Disco

---

## Estados visuais

### Loading
Mostrar skeleton simples ou labels como:
- `Coletando métricas...`
- barras em estado neutro

### Erro de backend
Mostrar card de erro curto:
- `Não foi possível atualizar as métricas`
- ação futura: `Tentar novamente`

### Sem dados parciais
Se uma seção não tiver dados:
- manter card
- mostrar `Não disponível`
- não quebrar o layout geral

---

## Comportamento do popup

### Abertura
- ao clicar no item do painel
- ancorado pelo próprio Plasma
- foco visual no popup sem janela tradicional do GTK

### Fechamento
- clicar fora
- ESC
- clicar novamente no item do painel

### Atualização
- atualização contínua sem recriar o layout inteiro
- apenas bindings/props devem mudar

### Desempenho
- evitar animações excessivas
- animações curtas apenas em barras, se necessário

---

## Tema e estilo

### Direção visual recomendada
**Breeze/KDE-inspired**, não Apple-like.

### Motivo
- integra melhor com Plasma
- reduz estranheza visual
- melhora acessibilidade em tema claro/escuro
- exige menos overrides de estilo

### Regras de estilo
- usar componentes Kirigami/Plasma sempre que possível
- evitar blur e transparência pesada na primeira versão
- usar cores semânticas do tema ou uma paleta mínima compatível
- espaçamento consistente (ex.: 8/12/16 px)

---

## Contrato mínimo esperado do backend

A UI precisará, no mínimo, destes dados:

### CPU
- uso total
- frequência
- nome
- quantidade de núcleos
- uso por núcleo

### Memória
- total
- usada
- disponível
- percentual
- swap total/usada

### Disco
- total agregado
- usado
- disponível
- percentual agregado
- lista por partição

### Rede
- total RX/TX
- lista de interfaces com RX/TX

### Sistema
- uptime
- load average

---

## Sequência de Execução

### 1. Definir tokens e componentes base
**Arquivos:** `MetricCard.qml`, `MetricBar.qml`, `MetricRow.qml`, `SectionHeader.qml`
**O que fazer:** criar os blocos reutilizáveis da interface.
**Dependências:** nenhuma

### 2. Implementar representação compacta do painel
**Arquivos:** `CompactRepresentation.qml`
**O que fazer:** criar ícone minimalista com CPU e RAM.
**Dependências:** passo 1

### 3. Montar estrutura do popup
**Arquivos:** `FullRepresentation.qml`
**O que fazer:** criar cabeçalho, cards e layout principal.
**Dependências:** passo 1

### 4. Implementar card de CPU
**Arquivos:** `FullRepresentation.qml`, `MetricBar.qml`, `MetricRow.qml`
**O que fazer:** mostrar uso total e uso por núcleo.
**Dependências:** passo 3

### 5. Implementar card de Memória
**Arquivos:** `FullRepresentation.qml`
**O que fazer:** mostrar RAM, disponível e swap.
**Dependências:** passo 3

### 6. Implementar card de Disco
**Arquivos:** `FullRepresentation.qml`
**O que fazer:** mostrar uso agregado e partições.
**Dependências:** passo 3

### 7. Implementar card de Rede
**Arquivos:** `FullRepresentation.qml`
**O que fazer:** mostrar RX/TX total e interfaces.
**Dependências:** passo 3

### 8. Implementar card de Sistema
**Arquivos:** `FullRepresentation.qml`
**O que fazer:** mostrar uptime e load average.
**Dependências:** passo 3

### 9. Conectar dados reais
**Arquivos:** `main.qml`, `FullRepresentation.qml`, backend DBus
**O que fazer:** trocar mocks/placeholders por dados reais do backend.
**Dependências:** passos 4 a 8

---

## Riscos e Mitigações

| Risco | Probabilidade | Mitigação |
|---|---|---|
| Popup ficar muito denso | média | limitar texto por card e usar grids compactos |
| Muitos núcleos poluírem a UI | alta | usar grid responsiva e limitar primeira exibição |
| Informações de rede ficarem longas | alta | quebrar linhas e truncar labels |
| Estilo fugir do tema KDE | média | priorizar Kirigami/Plasma Components |
| DBus atrasar a montagem da UI | média | desenvolver a UI com dados mockados primeiro |

---

## Critérios de Conclusão

- [ ] O item do painel permanece legível em tamanhos pequenos
- [ ] O popup usa componentes compatíveis com o tema do KDE
- [ ] CPU, memória, disco, rede e sistema possuem cards próprios
- [ ] CPU mostra uso por núcleo
- [ ] Disco mostra partições
- [ ] Rede mostra interfaces
- [ ] O layout continua legível em tema claro e escuro
- [ ] A UI consegue funcionar com placeholders antes da integração final com DBus
