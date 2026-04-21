#!/usr/bin/env bash
# Publica as páginas do diretório wiki/ no GitHub Wiki do repositório.
# Funciona tanto na primeira vez (wiki vazio) quanto em atualizações.
set -euo pipefail

REPO="marcos2872/rust-monitor-tray"
WIKI_URL="https://github.com/${REPO}.wiki.git"
WIKI_DIR="$(dirname "$0")/wiki"
TMP_DIR="$(mktemp -d)"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

echo "📄 Preparando páginas do wiki..."

# Tenta clonar; se o wiki ainda não existir, inicializa um repo novo
if git clone "$WIKI_URL" "$TMP_DIR" 2>/dev/null; then
  echo "✅ Wiki existente clonado."
else
  echo "ℹ️  Wiki ainda não existe — inicializando repositório novo."
  git init "$TMP_DIR"
  cd "$TMP_DIR"
  git remote add origin "$WIKI_URL"
  git checkout -b master
  cd - > /dev/null
fi

echo "📄 Copiando páginas..."
cp "$WIKI_DIR"/*.md "$TMP_DIR/"

cd "$TMP_DIR"
git add -A

if git diff --cached --quiet; then
  echo "ℹ️  Nenhuma mudança para publicar."
  exit 0
fi

git -c user.email="wiki@monitortray" -c user.name="Monitor Tray" \
  commit -m "docs(wiki): update pages from wiki/ directory"

echo "📤 Enviando para o GitHub Wiki..."
git push -u origin master 2>/dev/null || git push -u origin main

echo ""
echo "✅ Wiki publicado: https://github.com/${REPO}/wiki"
