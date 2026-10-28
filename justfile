# Tareas del proyecto. `just` lista todo; `just <tarea>` ejecuta una.

# UI HTML original, solo como referencia visual.
original := "/home/projects/ia/wacom/app"

# Lista las tareas disponibles.
default:
    @just --list

# Abre Tablet Center (debug) contra el tabletd instalado.
dev:
    cargo run -p tablet-center

# Abre Tablet Center con datos simulados: sin daemon ni escritura de configuración.
# `just dev-sim 0314` simula otro modelo (id USB en hex, p. ej. 0314 = PTH-451).
dev-sim product="":
    cargo run -p tablet-center -- --simulate{{ if product == "" { "" } else { "=" + product } }}

# Corre un tabletd local (detén antes tabletd.service si está activo).
daemon:
    cargo run -p tabletd

# Abre Tablet Center en release.
run:
    cargo run -p tablet-center --release

# Tests de todo el workspace.
test:
    cargo test --workspace

# Formato y clippy, como antes de cada commit (falla si hay algo).
lint:
    cargo fmt -p tablet-center --check
    cargo clippy --workspace --all-targets -- -D warnings

# Formatea el código.
fix:
    cargo fmt -p tablet-center

# Todo lo que debe estar en verde antes de commitear.
check: lint test

# Capturas PNG de la app egui en target/screens (necesita GPU).
screens:
    cargo test -p tablet-center --test screenshots -- --ignored

# Compara capturas con docs/reference en target/compare (sin nombres: todas).
compare *names:
    tools/compare.sh {{ if names == "" { "--all" } else { names } }}

# Capturas y comparación de todo, de una vez.
review: screens (compare)

# Regenera docs/reference desde la UI HTML original (Chromium + bun).
reference:
    #!/bin/sh
    set -eu
    if ! curl -s -o /dev/null http://127.0.0.1:8765/index.html; then
        bun {{original}}/tools/dev-server.ts & server=$!
        trap 'kill $server' EXIT
        sleep 1
    fi
    tools/capture-reference.sh

# Capturas extra de Settings del original (selects, toasts, diálogos).
reference-settings:
    node tools/capture-settings-reference.mjs

# Prueba las ventanas en pantalla en un servidor X aislado.
smoke-shortcuts:
    python3 tools/smoke-shortcuts.py

# Rehace las fuentes Roboto embebidas desde los WOFF2 del original.
fonts:
    python3 tools/rebuild-fonts.py

# Build de release de tablet-center y tabletd.
build:
    cargo build --release -p tablet-center -p tabletd

# Instala para el usuario actual (servicio, reglas udev, extensión de GNOME).
install:
    ./install.sh

# Desinstala; `just uninstall --purge` borra también la configuración.
uninstall *args:
    ./uninstall.sh {{args}}

# Genera dist/tablet-center-<arch>-linux.tar.gz.
package:
    scripts/package.sh

# Borra los artefactos de compilación.
clean:
    cargo clean
