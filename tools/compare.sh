#!/bin/sh
# Pone lado a lado docs/reference/<nombre>.png (izquierda) y target/screens/<nombre>.png
# (derecha), más un mapa de diferencias, en target/compare/<nombre>.png.
# Uso: tools/compare.sh <nombre> [...]   |   tools/compare.sh --all
set -eu
cd "$(dirname "$0")/.."
mkdir -p target/compare
if [ "${1:-}" = "--all" ]; then
  set -- $(ls target/screens/*.png 2>/dev/null | xargs -n1 basename | sed 's/\.png$//')
fi
for n in "$@"; do
  ref="${REFERENCE_DIR:-docs/reference}/$n.png"; got="target/screens/$n.png"
  [ -f "$ref" ] || { echo "sin referencia: $n"; continue; }
  [ -f "$got" ] || { echo "sin captura: $n"; continue; }
  diff=$(magick compare -metric RMSE "$ref" "$got" "target/compare/$n-diff.png" 2>&1 >/dev/null || true)
  magick "$ref" "$got" "target/compare/$n-diff.png" -resize 1100x +append "target/compare/$n.png"
  echo "$n  RMSE $diff"
done
