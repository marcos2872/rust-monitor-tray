#!/bin/bash

# Script para gerar .deb SEM autostart

echo "Gerando pacote .deb sem autostart..."

# 1. Fazer build de release
echo "🔨 Compilando aplicação..."
cargo build --release

# 2. Criar estrutura do pacote
echo "📁 Criando estrutura do pacote..."
mkdir -p releases/monitor-tray-deb/DEBIAN
mkdir -p releases/monitor-tray-deb/usr/bin
mkdir -p releases/monitor-tray-deb/usr/share/applications

# 3. Copiar executável
echo "📋 Copiando executável..."
cp target/release/monitor-tray releases/monitor-tray-deb/usr/bin/

# 4. Criar arquivo de controle
echo "📝 Criando arquivo de controle..."
cat > releases/monitor-tray-deb/DEBIAN/control << 'EOF'
Package: monitor-tray
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-3-0, libglib2.0-0, libappindicator3-1
Maintainer: Seu Nome <seu.email@example.com>
Description: System Monitor Tray Application
 A lightweight system tray application that displays real-time CPU and RAM usage.
 Features dynamic color-coded text display and updates every 500ms.
 Compatible with Unity, GNOME, and other desktop environments that support
 system tray indicators.
 .
 This version does not include automatic startup - launch manually or configure
 autostart through your desktop environment's settings.
EOF

# 5. Criar desktop entry APENAS para menu de aplicações
echo "🖥️ Criando entrada do menu..."
cat > releases/monitor-tray-deb/usr/share/applications/monitor-tray.desktop << 'EOF'
[Desktop Entry]
Name=Monitor Tray
Name[pt_BR]=Monitor da Bandeja
Comment=System monitor with CPU and RAM display in system tray
Comment[pt_BR]=Monitor de sistema com exibição de CPU e RAM na bandeja do sistema
Exec=/usr/bin/monitor-tray
Icon=utilities-system-monitor
Terminal=false
Type=Application
Categories=System;Monitor;Utility;
Keywords=system;monitor;cpu;ram;memory;tray;indicator;
StartupNotify=false
StartupWMClass=monitor-tray
EOF

# 6. Definir permissões corretas
echo "🔐 Definindo permissões..."
chmod 755 releases/monitor-tray-deb/usr/bin/monitor-tray
chmod 644 releases/monitor-tray-deb/usr/share/applications/monitor-tray.desktop

# 7. Criar script pós-instalação
echo "📜 Criando scripts de instalação..."
cat > releases/monitor-tray-deb/DEBIAN/postinst << 'EOF'
#!/bin/bash
set -e

# Atualizar cache de desktop entries
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications
fi

echo "✅ Monitor Tray instalado com sucesso!"
echo ""
echo "📍 Como usar:"
echo "  • Executar: monitor-tray (ou procure 'Monitor Tray' no menu)"
echo "  • Para finalizar: clique com botão direito no ícone → Sair"
echo ""
echo "🚀 Para configurar inicialização automática:"
echo "  • GNOME: Configurações → Aplicações → Aplicações de Inicialização"
echo "  • KDE: Configurações do Sistema → Inicialização → Autostart"
echo "  • XFCE: Configurações → Sessão e Inicialização → Autostart"
echo "  • Comando: monitor-tray"
echo ""

exit 0
EOF

# 8. Criar script pré-remoção
cat > releases/monitor-tray-deb/DEBIAN/prerm << 'EOF'
#!/bin/bash
set -e

# Parar aplicação se estiver rodando
if pgrep -x "monitor-tray" > /dev/null; then
    echo "🛑 Parando Monitor Tray..."
    pkill -x "monitor-tray" || true
    sleep 1
fi

exit 0
EOF

# 9. Criar script pós-remoção
cat > releases/monitor-tray-deb/DEBIAN/postrm << 'EOF'
#!/bin/bash
set -e

# Atualizar cache de desktop entries
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications
fi

# Remover arquivos temporários se existirem
rm -f /tmp/monitor_text_icon*.svg 2>/dev/null || true

echo "📦 Monitor Tray removido com sucesso!"

exit 0
EOF

# 10. Definir permissões dos scripts
chmod 755 releases/monitor-tray-deb/DEBIAN/postinst
chmod 755 releases/monitor-tray-deb/DEBIAN/prerm
chmod 755 releases/monitor-tray-deb/DEBIAN/postrm

# 11. Gerar o pacote .deb
echo "📦 Gerando pacote .deb..."
dpkg-deb --build releases/monitor-tray-deb

# 12. Informações finais
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ PACOTE GERADO COM SUCESSO!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📂 Arquivo: releases/monitor-tray-deb.deb"
echo "📊 Tamanho: $(du -h releases/monitor-tray-deb.deb | cut -f1)"
echo ""
echo "🔧 INSTALAÇÃO:"
echo "  sudo dpkg -i releases/monitor-tray-deb.deb"
echo "  sudo apt-get install -f  # (se houver problemas de dependências)"
echo ""
echo "🚀 EXECUÇÃO:"
echo "  monitor-tray  # (via terminal)"
echo "  # OU procure 'Monitor Tray' no menu de aplicações"
echo ""
echo "⚙️ CONFIGURAR AUTOSTART (opcional):"
echo "  • GNOME: Settings → Applications → Startup Applications"
echo "  • KDE: System Settings → Startup and Shutdown → Autostart"
echo "  • XFCE: Settings → Session and Startup → Application Autostart"
echo "  • Comando a adicionar: /usr/bin/monitor-tray"
echo ""
echo "🗑️ DESINSTALAÇÃO:"
echo "  sudo apt remove monitor-tray"
echo ""
echo "════════════════════════════════════════════════════════════════"

# 13. Verificação do pacote (opcional)
if command -v dpkg-deb >/dev/null 2>&1; then
    echo "🔍 INFORMAÇÕES DO PACOTE:"
    dpkg-deb -I releases/monitor-tray-deb.deb
    echo ""
    echo "📋 CONTEÚDO DO PACOTE:"
    dpkg-deb -c releases/monitor-tray-deb.deb
fi
