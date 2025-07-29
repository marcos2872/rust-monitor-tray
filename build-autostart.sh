#!/bin/bash

# Script completo para gerar .deb com autostart

echo "Gerando pacote .deb com autostart..."

# 1. Fazer build de release
cargo build --release

# 2. Criar estrutura do pacote
mkdir -p releases/monitor-tray-autostart-deb/DEBIAN
mkdir -p releases/monitor-tray-autostart-deb/usr/bin
mkdir -p releases/monitor-tray-autostart-deb/usr/share/applications
mkdir -p releases/monitor-tray-autostart-deb/etc/xdg/autostart

# 3. Copiar executável
cp target/release/monitor-tray releases/monitor-tray-autostart-deb/usr/bin/

# 4. Criar arquivo de controle
cat > releases/monitor-tray-autostart-deb/DEBIAN/control << 'EOF'
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
 system tray indicators. Includes automatic startup configuration.
EOF

# 5. Criar desktop entry para menu de aplicações
cat > releases/monitor-tray-autostart-deb/usr/share/applications/monitor-tray.desktop << 'EOF'
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
EOF

# 6. Criar autostart entry (será copiado para ~/.config/autostart/ pelo usuário)
cat > releases/monitor-tray-autostart-deb/etc/xdg/autostart/monitor-tray.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Monitor Tray
Name[pt_BR]=Monitor da Bandeja
Comment=System monitor with CPU and RAM display in system tray
Comment[pt_BR]=Monitor de sistema com exibição de CPU e RAM na bandeja do sistema
Exec=/usr/bin/monitor-tray
Icon=utilities-system-monitor
Terminal=false
Categories=System;Monitor;
StartupNotify=false
Hidden=false
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=3
X-KDE-autostart-after=panel
NoDisplay=false
EOF

# 7. Definir permissões corretas
chmod 755 releases/monitor-tray-autostart-deb/usr/bin/monitor-tray
chmod 644 releases/monitor-tray-autostart-deb/usr/share/applications/monitor-tray.desktop
chmod 644 releases/monitor-tray-autostart-deb/etc/xdg/autostart/monitor-tray.desktop

# 8. Criar script pós-instalação (opcional)
cat > releases/monitor-tray-autostart-deb/DEBIAN/postinst << 'EOF'
#!/bin/bash
set -e

# Atualizar cache de desktop entries
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications
fi

# Informar usuário sobre autostart
echo "Monitor Tray instalado com sucesso!"
echo ""
echo "O aplicativo foi configurado para iniciar automaticamente."
echo "Para desabilitar o autostart:"
echo "  - GNOME: Settings > Applications > Startup Applications"
echo "  - KDE: System Settings > Startup and Shutdown > Autostart"
echo "  - Ou remova: ~/.config/autostart/monitor-tray.desktop"
echo ""
echo "Para iniciar agora: monitor-tray"

exit 0
EOF

# 9. Criar script pré-remoção (opcional)
cat > releases/monitor-tray-autostart-deb/DEBIAN/prerm << 'EOF'
#!/bin/bash
set -e

# Parar aplicação se estiver rodando
if pgrep -x "monitor-tray" > /dev/null; then
    echo "Parando Monitor Tray..."
    pkill -x "monitor-tray" || true
fi

exit 0
EOF

# 10. Definir permissões dos scripts
chmod 755 releases/monitor-tray-autostart-deb/DEBIAN/postinst
chmod 755 releases/monitor-tray-autostart-deb/DEBIAN/prerm

# 11. Gerar o pacote .deb
dpkg-deb --build releases/monitor-tray-autostart-deb

echo ""
echo "✅ Pacote gerado: releases/monitor-tray-autostart-deb.deb"
echo ""
echo "Para instalar:"
echo "  sudo dpkg -i releases/monitor-tray-autostart-deb.deb"
echo "  sudo apt-get install -f  # se houver problemas de dependências"
echo ""
echo "Para testar o autostart:"
echo "  1. Instale o pacote"
echo "  2. Faça logout/login"
echo "  3. Verifique se apareceu na system tray"
echo ""
echo "Para desinstalar:"
echo "  sudo apt remove monitor-tray"
