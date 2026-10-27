#!/bin/sh
# Captura la UI HTML original (wacom/app/ui, servida por su dev-server en :8765)
# con Chromium headless. Salida: docs/reference/<estilo>-<tema>-<ruta>.png
set -eu
out="$(cd "$(dirname "$0")/.." && pwd)/docs/reference"
mkdir -p "$out"
base=http://127.0.0.1:8765
shot() { # archivo url tema alto
  flags=""
  [ "$3" = dark ] && flags="--force-dark-mode"
  chromium --headless=new --disable-gpu --hide-scrollbars $flags \
    --window-size=1100,$4 --virtual-time-budget=4000 \
    --screenshot="$out/$1.png" "$2" >/dev/null 2>&1
}
classic="devices devices/pen devices/expresskeys devices/mapping devices/display-toggle devices/info devices/touch
shortcuts shortcuts/express-menu shortcuts/radial-menu shortcuts/app-shortcuts shortcuts/keypad
shortcuts/pan-scroll shortcuts/pan-scroll-legacy shortcuts/pan-zoom shortcuts/scroll shortcuts/wheel shortcuts/zoom
settings/general settings/devices settings/backups"
for theme in dark light; do
  for r in $classic; do
    n=$(echo "$r" | tr / -)
    shot "classic-$theme-$n" "$base/index.html#/$r" $theme 720
    shot "classic-$theme-$n-tall" "$base/index.html#/$r" $theme 1600
  done
done
echo "listo: $out"
