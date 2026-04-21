#!/usr/bin/env bash
set -euo pipefail

APP_NAME="monitor-tray"
PLASMOID_ID="com.monitortray.plasmoid"
BIN_DIR="${HOME}/.local/bin"
BIN_PATH="${BIN_DIR}/${APP_NAME}"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_PATH="${SERVICE_DIR}/${APP_NAME}.service"
RELOAD_PLASMA=1

usage() {
  cat <<EOF
Remove o monitor-tray instalado para KDE/Plasma no usuário atual.

Uso:
  ./uninstall-kde.sh [--no-plasma-reload] [--bin-dir <diretório>]

Opções:
  --no-plasma-reload   Não reinicia o plasmashell ao final
  --bin-dir DIR        Diretório onde o binário foi instalado (padrão: ~/.local/bin)
  -h, --help           Exibe esta ajuda
EOF
}

find_kpackagetool() {
  if command -v kpackagetool6 >/dev/null 2>&1; then
    echo "kpackagetool6"
  elif command -v kpackagetool5 >/dev/null 2>&1; then
    echo "kpackagetool5"
  else
    echo ""
  fi
}

reload_plasma() {
  if [[ "$RELOAD_PLASMA" -eq 0 ]]; then
    return 0
  fi

  if ! command -v plasmashell >/dev/null 2>&1; then
    echo "Aviso: plasmashell não encontrado; pulando reload do Plasma."
    return 0
  fi

  echo "↻ Recarregando plasmashell..."
  kquitapp6 plasmashell >/dev/null 2>&1 || kquitapp5 plasmashell >/dev/null 2>&1 || true
  nohup plasmashell >/dev/null 2>&1 &
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-plasma-reload)
      RELOAD_PLASMA=0
      shift
      ;;
    --bin-dir)
      if [[ $# -lt 2 ]]; then
        echo "Erro: --bin-dir exige um valor" >&2
        exit 1
      fi
      BIN_DIR="$2"
      BIN_PATH="${BIN_DIR}/${APP_NAME}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Erro: opção inválida: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

KPACKAGETOOL="$(find_kpackagetool)"
if [[ -z "$KPACKAGETOOL" ]]; then
  echo "Erro: kpackagetool5 ou kpackagetool6 não encontrado." >&2
  exit 1
fi

if command -v systemctl >/dev/null 2>&1; then
  echo "🛑 Desativando serviço systemd do usuário..."
  systemctl --user disable --now "${APP_NAME}.service" >/dev/null 2>&1 || true
  systemctl --user daemon-reload || true
else
  echo "Aviso: systemctl não encontrado; pulando remoção do serviço user."
fi

if [[ -f "$SERVICE_PATH" ]]; then
  echo "🗑️ Removendo serviço em $SERVICE_PATH..."
  rm -f "$SERVICE_PATH"
  if command -v systemctl >/dev/null 2>&1; then
    systemctl --user daemon-reload || true
  fi
else
  echo "ℹ️ Serviço user não encontrado em $SERVICE_PATH"
fi

if [[ -f "$BIN_PATH" ]]; then
  echo "🗑️ Removendo binário em $BIN_PATH..."
  rm -f "$BIN_PATH"
else
  echo "ℹ️ Binário não encontrado em $BIN_PATH"
fi

echo "🧩 Removendo plasmoid $PLASMOID_ID..."
"$KPACKAGETOOL" --type Plasma/Applet --remove "$PLASMOID_ID" >/dev/null 2>&1 || true

echo "🖼️  Removendo ícone do tema hicolor do usuário..."
ICON_PATH="${HOME}/.local/share/icons/hicolor/256x256/apps/com.monitortray.plasmoid.png"
rm -f "$ICON_PATH"
if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
  kbuildsycoca5 --noincremental >/dev/null 2>&1 || true
fi

reload_plasma

cat <<EOF

✅ Remoção concluída.

Resumo:
- Binário removido: $BIN_PATH
- Serviço removido: $SERVICE_PATH
- Plasmoid removido: $PLASMOID_ID

Verificações úteis:
  systemctl --user status ${APP_NAME}.service
  $KPACKAGETOOL --type Plasma/Applet --list | grep ${PLASMOID_ID}
EOF
