# Monitor Tray

Um aplicativo leve de monitoramento de sistema que exibe o uso de CPU e RAM em tempo real na system tray (bandeja do sistema). Desenvolvido em Rust para máxima performance e baixo consumo de recursos.

## 📋 Características

- ✅ **Monitoramento em tempo real** - CPU e RAM atualizados a cada 500ms
- ✅ **Interface visual dinâmica** - Cores que mudam baseadas no uso do sistema:
  - 🟢 **Branco**: Uso baixo (< 50%)
  - 🟡 **Amarelo**: Uso médio (50-80%)
  - 🔴 **Vermelho**: Uso alto (> 80%)
- ✅ **Font bold** para melhor legibilidade
- ✅ **Sistema tray nativo** - Integração perfeita com o desktop
- ✅ **Menu contextual** com opção de sair
- ✅ **Compatibilidade ampla** - Unity, GNOME, KDE e outros ambientes desktop Linux
- ✅ **Baixo consumo de recursos** - Escrito em Rust para máxima eficiência

## 🖥️ Interface

O aplicativo exibe na system tray:
```
CPU  RAM
15%  8.2gb
```

As cores dos valores mudam dinamicamente conforme o uso do sistema.

## 📦 Instalação

### Via pacote .deb (Ubuntu/Debian)

1. Baixe o arquivo `.deb` da seção [Releases](releases)
2. Instale o pacote:
```bash
sudo dpkg -i monitor-tray-deb.deb
sudo apt-get install -f  # instala dependências se necessário
```

### Dependências do sistema
O aplicativo requer as seguintes bibliotecas:
- `libgtk-3-0`
- `libglib2.0-0`
- `libappindicator3-1`

## 🚀 Execução

Após a instalação, você pode:

1. **Executar via linha de comando:**
```bash
monitor-tray
```

2. **Executar via menu de aplicações:**
Procure por "Monitor Tray" no launcher de aplicações

3. **Inicialização automática:**
O aplicativo está configurado para aparecer nas opções de inicialização automática do sistema

## 🛠️ Desenvolvimento

### Pré-requisitos

- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- Dependências de desenvolvimento do GTK:

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential pkg-config libgtk-3-dev libglib2.0-dev libappindicator3-dev
```

**Fedora:**
```bash
sudo dnf install gcc pkg-config gtk3-devel glib2-devel libappindicator-gtk3-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel pkgconf gtk3 glib2 libappindicator-gtk3
```

### Clone e execução

```bash
# Clone o repositório
git clone <repository-url>
cd monitor-tray

# Execute em modo de desenvolvimento
cargo run

# Ou execute com logs detalhados
RUST_LOG=debug cargo run
```

### Build de produção

```bash
# Gerar executável otimizado
cargo build --release

# O executável estará em:
# ./target/release/monitor-tray
```

### Gerar pacote .deb

```bash
# 1. Fazer build de release
cargo build --release

# 2. Criar estrutura do pacote
mkdir -p monitor-tray-deb/DEBIAN
mkdir -p monitor-tray-deb/usr/bin
mkdir -p monitor-tray-deb/usr/share/applications

# 3. Copiar arquivos
cp target/release/monitor-tray releases/monitor-tray-deb/usr/bin/

# 4. Criar arquivo de controle
cat > releases/monitor-tray-deb/DEBIAN/control << EOF
Package: monitor-tray
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-3-0, libglib2.0-0, libappindicator3-1
Maintainer: Marcos <marcos@example.com>
Description: System Monitor Tray Application
 A lightweight system tray application that displays real-time CPU and RAM usage.
 Features dynamic color-coded text display and updates every 500ms.
 Compatible with Unity, GNOME, and other desktop environments that support
 system tray indicators.

EOF

# 5. Criar desktop entry
cat > releases/monitor-tray-deb/usr/share/applications/monitor-tray.desktop << EOF
[Desktop Entry]
Name=Monitor Tray
Comment=System monitor with CPU and RAM display in system tray
Exec=/usr/bin/monitor-tray
Icon=utilities-system-monitor
Terminal=false
Type=Application
Categories=System;Monitor;
StartupNotify=false
X-GNOME-Autostart-enabled=true
EOF

# 6. Definir permissões
chmod 755 releases/monitor-tray-deb/usr/bin/monitor-tray
chmod 644 releases/monitor-tray-deb/usr/share/applications/monitor-tray.desktop

# 7. Gerar o pacote .deb
dpkg-deb --build releases/monitor-tray-deb

# O arquivo monitor-tray-deb.deb será criado
```

### Estrutura do projeto

```
monitor-tray/
├─ src/
│  └─ main.rs          # Código principal
├─ Cargo.toml          # Dependências e configuração
├─ README.md           # Este arquivo
└─ target/
   └─ release/
      └─ monitor-tray  # Executável otimizado
```

## 🔧 Tecnologias utilizadas

- **[Rust](https://www.rust-lang.org/)** - Linguagem de programação
- **[sysinfo](https://crates.io/crates/sysinfo)** - Coleta de informações do sistema
- **[libappindicator](https://crates.io/crates/libappindicator)** - Sistema tray no Linux
- **[GTK](https://www.gtk.org/)** - Interface gráfica e menus
- **SVG** - Geração dinâmica de ícones

## 🐛 Resolução de problemas

### O ícone não aparece na system tray
- Verifique se seu ambiente desktop suporta system tray
- No GNOME, instale a extensão "TopIcons Plus" ou "AppIndicator Support"
- Reinicie o aplicativo após instalar extensões

### Erro de dependências
```bash
# Instale as dependências manualmente
sudo apt install libgtk-3-0 libglib2.0-0 libappindicator3-1
```

### Problemas de compilação
- Verifique se todas as dependências de desenvolvimento estão instaladas
- Atualize o Rust: `rustup update`

## 📄 Licença

Este projeto está sob a licença MIT. Veja o arquivo `LICENSE` para mais detalhes.

## 🤝 Contribuição

Contribuições são bem-vindas! Sinta-se à vontade para:

1. Fazer fork do projeto
2. Criar uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abrir um Pull Request

## 📞 Suporte

Se você encontrar algum problema ou tiver sugestões, por favor:
- Abra uma [issue](issues) no GitHub
- Descreva o problema detalhadamente
- Inclua informações do sistema (distribuição, versão, ambiente desktop)

---

**Monitor Tray** - Monitoramento de sistema simples e eficiente para Linux 🐧
