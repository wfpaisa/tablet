# AGENTS.md

Proyecto **Tablet Center (Rust puro)**: port de la app Tauri en
`/home/projects/ia/wacom/app` (solo lectura) a una app nativa **egui/eframe, sin
Tauri ni WebView**, al estilo de `/home/projects/plane-llama-rust` (solo lectura).
Leer `PLAN.md` antes de empezar y hacer solo la fase que se pida.

## Fuente de verdad

- **Aspecto**: `docs/reference/*.png` son capturas de la UI HTML original a
  1100×720 (y `-tall` a 1100×1600). La app egui debe verse **igual**: mismas
  posiciones, tamaños, colores, radios, tipografía, iconos, textos e ilustraciones.
  Los valores exactos salen de `ui/style.css`, `ui/frame.css`, `ui/select.css`,
  `ui/glass.css` del original: copiarlos, no inventar.
- **Comportamiento**: `ui/app.js`, `ui/glass.js`, `ui/shortcut.js` y
  `crates/tablet-center/src/main.rs` del original. Cada comando Tauri pasa a ser
  una función Rust normal llamada desde la UI (en un hilo aparte si toca disco,
  red o el daemon, para no congelar la ventana).
- `crates/tabletd` y `crates/tablet-config` ya están copiados y **no se tocan**
  salvo lo imprescindible para compilar. El protocolo con tabletd es el mismo.

## Reglas

- Stack: `eframe`/`egui` 0.36 (wgpu + wayland + x11), `egui_extras` con `svg` e
  `image` para iconos e imágenes de `assets/`, `egui_kittest` (wgpu) para capturas.
  Fuentes de `assets/fonts` (Roboto), embebidas
  con `include_bytes!`. Para ja/ko/zh cargar una fuente CJK del sistema si existe.
- Ventana sin decoraciones (`with_decorations(false)`), barra de título propia como
  en las capturas (icono, «Tablet Center», botones ─ ☐ ✕ redondos), arrastre,
  doble clic maximiza, bordes redimensionan. Borde oscuro con brillo interior como
  `frame.css`.
- Todos los colores, radios y medidas en un módulo `theme.rs` (tokens), nada de
  colores literales sueltos por la UI.
- Textos traducidos con `assets/locales/*.json` (misma lógica que `t()` en app.js,
  incluido el «debrand» de la interfaz clásica). Inglés por defecto.
- Identificadores en inglés; comentarios cortos en inglés como el original (cada
  módulo con un encabezado que diga su responsabilidad).
- Nada de `unwrap()`/`expect()` fuera de tests y `main`.
- Usar las tareas del `justfile` (ver `docs/development.md`); si hace falta una
  tarea nueva, agregarla al `justfile` y a `docs/development.md`.
- Antes de commitear: `just fix` y `just check` (fmt, clippy con `-D warnings`
  y tests). Todo en verde.
- No tocar nada fuera de `/home/projects/tablet`. No instalar ni reiniciar el
  servicio `tabletd` del sistema; para probar con el daemon real basta leer su
  socket. No escribir en `~/.config/tablet-center` (usar un directorio temporal
  en tests y en capturas).

## Capturas y comparación (obligatorio en cada fase)

- `just screens` genera
  `target/screens/<nombre>.png` a 1100×720 (pixels_per_point 1) con los **mismos
  datos simulados** que usa `app.js` en navegador (Intuos S 0x0374, dos monitores
  Dell 2560×1440 y 1920×1200, DEFAULTS). Los nombres siguen a
  `docs/reference/` (p. ej. `classic-dark-devices-pen.png`).
- `just compare <nombre>` (o `just review` para todo) pone lado a lado referencia y captura en
  `target/compare/<nombre>.png`. Mirar esas imágenes (con la herramienta de ver
  imágenes) y corregir hasta que no se note diferencia.
- Al terminar cada fase: commit `feat(fase-N): ...` y resumen en
  `docs/progress.md` (hecho, pendiente, diferencias visuales conocidas).
