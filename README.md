# System Monitor Desktop Bar

Uma barra de monitoramento de sistema elegante e minimalista que fica sempre visível no topo da tela. Desenvolvida em Rust para máxima performance e baixo consumo de recursos.

## ✨ Características

- 🖥️ **Barra desktop sempre no topo** - Fica sobreposta a todas as janelas
- 📊 **Monitoramento em tempo real** - Atualizações a cada segundo
- 🎨 **Interface elegante** com fundo semi-transparente e bordas arredondadas
- 🌈 **Cores dinâmicas** baseadas no uso do sistema:
  - 🟢 **Aquamarine**: Uso baixo (< 50%)
  - 🟡 **Dourado**: Uso médio (50-80%)
  - 🔴 **Coral**: Uso alto (> 80%)
- 📐 **Layout fixo** - Textos não se movem durante atualizações
- 📍 **Posicionamento customizável** - Canto superior esquerdo por padrão
- 💾 **Informações exibidas**:
  - **CPU**: Porcentagem de uso
  - **RAM**: Memória utilizada em GB
  - **Rede**: Tráfego de download/upload total
  - **Uptime**: Tempo de atividade do sistema
- ⚡ **Ultra compacta** - Apenas 250px × 32px
- 🎯 **Baixo consumo** - Interface leve e eficiente

## 🚀 Execução Rápida

```bash
# Clone o repositório
git clone <repository-url>
cd rust-monitor-tray

# Execute diretamente
cargo run
```

## 📦 Instalação

### Compilação e instalação manual

```bash
# 1. Clone o repositório
git clone <repository-url>
cd rust-monitor-tray

# 2. Compile em modo release
cargo build --release

# 3. Execute a barra desktop
./target/release/monitor-tray
```

### Dependências do sistema
O aplicativo requer as seguintes bibliotecas GTK:
- `libgtk-3-0`
- `libglib2.0-0`

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-3-0 libglib2.0-0
```

**Fedora:**
```bash
sudo dnf install gtk3 glib2
```

## 🛠️ Desenvolvimento

### Pré-requisitos

- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- Dependências de desenvolvimento do GTK:

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential pkg-config libgtk-3-dev libglib2.0-dev
```

**Fedora:**
```bash
sudo dnf install gcc pkg-config gtk3-devel glib2-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel pkgconf gtk3 glib2
```

### Desenvolvimento local

```bash
# Execute em modo de desenvolvimento
cargo run

# Execute com logs detalhados
RUST_LOG=debug cargo run

# Compile versão otimizada
cargo build --release
```

### Estrutura do projeto

```
rust-monitor-tray/
├─ src/
│  ├─ main.rs          # Interface desktop bar e lógica principal
│  └─ monitor.rs       # Módulo de coleta de métricas do sistema
├─ Cargo.toml          # Dependências e configuração
├─ README.md           # Este arquivo
└─ target/
   └─ release/
      └─ monitor-tray  # Executável otimizado
```

## 🔧 Tecnologias utilizadas

- **[Rust](https://www.rust-lang.org/)** - Linguagem de programação para máxima performance e segurança
- **[sysinfo](https://crates.io/crates/sysinfo)** - Coleta de métricas do sistema multiplataforma
- **[GTK 3](https://www.gtk.org/)** - Interface gráfica nativa do Linux
- **[tokio](https://tokio.rs/)** - Runtime assíncrono para atualizações em tempo real
- **CSS** - Estilização com gradientes, transparência e bordas arredondadas

## ⚙️ Personalização

### Posicionamento da barra
Edite `src/main.rs` na linha de posicionamento:
```rust
window.move_(10, 5); // x=10, y=5 (canto superior esquerdo)
window.move_(960, 5); // Centralizado em tela 1920px
```

### Transparência do fundo
Ajuste a opacidade no CSS:
```rust
background: rgba(0, 0, 0, 0.8); // 80% opaco
background: rgba(0, 0, 0, 0.3); // 30% opaco (mais transparente)
```

### Dimensões da barra
Modifique o tamanho:
```rust
.default_width(250)  // Largura em pixels
.default_height(32)  // Altura in pixels
```

## 🐛 Resolução de problemas

### A barra não aparece
- Verifique se as dependências GTK estão instaladas
- Teste com `cargo run` para ver mensagens de erro
- Alguns gerenciadores de janela podem bloquear janelas "always on top"

### Fundo completamente transparente
- Seu sistema pode não ter um compositor ativo (Picom, Compton, etc.)
- Instale um compositor: `sudo apt install picom`
- Ou ajuste para fundo sólido alterando o CSS para usar cores hex: `#000000`

### Problemas de compilação
- Instale as dependências de desenvolvimento: `sudo apt install build-essential pkg-config libgtk-3-dev`
- Atualize o Rust: `rustup update`

## 🚀 Próximas funcionalidades

- [ ] Suporte a múltiplos monitores
- [ ] Configuração via arquivo de configuração
- [ ] Temas personalizáveis
- [ ] Clique para expandir informações detalhadas
- [ ] Histórico de uso em gráficos
- [ ] Alertas por notificação

## 🤝 Contribuição

Contribuições são bem-vindas! Para contribuir:

1. Faça fork do projeto
2. Crie uma branch: `git checkout -b feature/nova-funcionalidade`
3. Commit suas mudanças: `git commit -m 'Adiciona nova funcionalidade'`
4. Push para a branch: `git push origin feature/nova-funcionalidade`
5. Abra um Pull Request

## 📞 Suporte

- 🐛 **Issues**: Relate bugs ou sugira melhorias
- 💬 **Discussões**: Ideias e dúvidas gerais
- 📧 **Email**: Para suporte direto

---

**System Monitor Desktop Bar** - Monitoramento elegante e minimalista para Linux 🐧✨
