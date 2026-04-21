#!/usr/bin/env bash
# Monitor Tray — instalador one-liner
# Uso: curl -fsSL https://raw.githubusercontent.com/marcos2872/rust-monitor-tray/main/install.sh | sh
set -euo pipefail

REPO="marcos2872/rust-monitor-tray"
BRANCH="main"
TARBALL_URL="https://github.com/${REPO}/archive/refs/heads/${BRANCH}.tar.gz"
TMP_DIR="$(mktemp -d)"
EXTRACT_DIR="${TMP_DIR}/rust-monitor-tray-${BRANCH}"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

echo "⬇️  Baixando monitor-tray (branch ${BRANCH})..."
curl -fsSL "$TARBALL_URL" | tar -xz -C "$TMP_DIR"

echo "🔨 Iniciando instalação..."
cd "$EXTRACT_DIR"
bash install-kde.sh "$@"
