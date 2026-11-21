# Plan: anillo estilo OpenLogi, azul #007DFF, desenfoque y aviso de modo

Implementación por etapas (una ejecución de codex por etapa). Al final de cada
etapa: `just fix`, `just check` en verde. No commitear (lo revisa el supervisor).
Reglas de `AGENTS.md`: colores y medidas solo en `theme.rs`, sin `unwrap()`,
comentarios cortos en inglés, textos con `assets/locales/*.json` (todas las
lenguas, inglés por defecto). Se permite tocar `tabletd` y la extensión de GNOME
para lo que pide este plan.

---

## Etapa 1 — Azul #007DFF en toda la app

- `theme.rs`: `ACCENT = #007DFF` (0, 125, 255). Recalcular **desde el nuevo
  acento** todos los tonos derivados del azul viejo #3584E4 (53,132,228):
  `DISABLED_ACCENT`, `BEAM`, `BEAM_EDGE`, `AREA`, `SCREEN`, `CHIP`, `FOCUS`,
  `TOGGLE_BEAM_ACTIVE`, y cualquier otro que sea mezcla/alpha del azul (buscar
  todos). Mantener la misma relación (mismo alpha, misma mezcla con el fondo).
  Mejor: definirlos como funciones/constantes derivadas de `ACCENT`.
- SVG/íconos que lleven el azul viejo embebido (buscar `3584e4`, `#3584E4`,
  `53, 132, 228`, `0.208, 0.518, 0.894`) en `crates/`, `assets/`,
  `gnome-extension/`. Cambiarlos todos.
- Extensión GNOME (`stylesheet.css`, `menus.js`, `extension.js`): mismo azul.
- Las capturas de referencia seguirán con el azul viejo: es esperado.

## Etapa 2 — Anillo (radial y Express menu) como OpenLogi

Referencia visual (imagen del usuario): fondo oscuro, **sin rosca continua**.

Geometría (a `theme.rs`, escalar con el tamaño configurado del menú;
valores para diámetro total ≈ 300 px):
- Cada acción es un **botón circular independiente** de Ø 56 px, centrado en
  un círculo de radio ≈ 105 px, en sentido horario desde arriba (mismo orden e
  índices que hoy, mismo hit‑test por ángulo para no romper nada).
- Relleno: gris neutro translúcido `rgba(58,58,58,0.88)`, borde 1 px
  `rgba(255,255,255,0.06)`, sombra suave. Solo **ícono** blanco 22 px, sin
  texto debajo.
- Centro: botón cerrar pequeño Ø 36 px, `rgba(90,90,90,0.85)`, «×» gris claro.
- **Hover/selección**: el botón crece a Ø 72 px (animado ~120 ms, ease‑out),
  relleno `#007DFF`, aro blanco 2 px a 2 px por fuera (como en la imagen),
  halo azul difuso alrededor. Un **haz** (cuña/triángulo) azul translúcido sale
  del botón central hacia el botón activo, con degradado (más opaco cerca del
  centro → transparente hacia el botón).
- **Etiqueta**: píldora blanca (`#FFFFFF`, texto `#1A1A1A`, 14 px, radio 8,
  padding 6×12) con flechita apuntando al botón, colocada en el lado exterior
  del botón activo (debajo si está en la mitad inferior, arriba si en la
  superior). Solo para el activo.
- Deshabilitados: ícono al 30 %, sin hover ni haz.
- Apertura: escala 0.9→1 + fundido 140 ms (como ya hace la extensión).
- El área transparente alrededor de los botones no es fondo: la ventana del
  overlay sigue siendo transparente.

Dónde:
1. egui: el dibujo compartido de `classic/shortcuts.rs` (vista previa en
   Shortcuts → Radial/Express menu, donde el ítem seleccionado se pinta como
   activo y muestra su píldora; y el overlay `overlay.rs`).
2. GNOME: `gnome-extension/.../menus.js` (`radialMenu`) y `stylesheet.css`,
   mismo diseño con Cairo + St (actores St por botón con transición de escala
   está bien).
3. Captura nueva `overlay-radial-menu.png` en `tests/screenshots.rs` (con un
   ítem en hover) y tarea en `just screens`.

## Etapa 3 — Desenfoque de fondo (GNOME, Wayland)

Portar `/home/projects/plane-llama/crates/ui/src/blur.rs` (solo lectura) a
`crates/tablet-center/src/blur.rs` (`ext_background_effect_v1`, GNOME 51+),
con las mismas dependencias (`wayland-client`, `wayland-backend` con
`client_system`/`dlopen`, `wayland-protocols` con `staging`, `raw-window-handle`).
Comentarios en inglés.

- `Blur::new` devuelve `None` fuera de Wayland o sin el protocolo;
  `supported()` según la capacidad anunciada.
- Ajuste persistente `blur` (por defecto **false**) donde la app guarda sus
  preferencias de interfaz (idioma/estilo). No en `~/.config` durante tests.
- Settings → General: fila con switch «Blur background» (+ ayuda corta).
  Si no está soportado: switch **desactivado (gris) y apagado**, con nota
  «Not supported by this desktop». Traducir a todas las lenguas.
- Activado: la ventana se crea `with_transparent(true)`; los fondos grandes
  (ventana, barra lateral, toolbar, tarjetas) pasan a variantes translúcidas
  (tokens `*_GLASS` en `theme.rs`, alpha ~0.72–0.82) para que se vea el
  desenfoque; el área desenfocada = rect interior de la ventana con el radio
  del marco (ver `apply_blur` en plane-llama `lib.rs`). Desactivado: todo
  idéntico a hoy (las capturas no deben cambiar).
- Opcional si sale limpio: los botones del anillo del overlay también con
  desenfoque detrás (regiones circulares).

## Etapa 4 — Aviso de cambio de modo (Pen ⇄ Mouse / Relative)

Cuando tabletd cambia el modo del lápiz (`toggle_mode` / `toggle_mouse`,
`crates/tabletd/src/main.rs` ~390), mostrar un aviso en el centro de la
pantalla, en la línea visual del anillo.

Diseño (tokens en `theme.rs`):
- Tarjeta redondeada (radio 28) ~ 320×150, `rgba(30,30,30,0.86)` (desenfocada
  si se puede), borde 1 px blanco 6 %.
- Dos círculos Ø 64 a izquierda y derecha: el modo anterior en gris
  translúcido (ícono al 50 %), el modo nuevo en `#007DFF` con aro blanco y halo
  (mismo estilo que el botón activo del anillo). Entre ellos una flecha → que
  se dibuja/desliza de izquierda a derecha.
- Debajo, texto «Pen → Mouse» (traducido) 15 px, y el nombre del modo nuevo en
  negrita.
- Íconos lucide: lápiz (`pen-line`/`pencil`), ratón (`mouse`), relativo
  (`move`/`mouse-pointer-2`).
- Animación: entra escala 0.9→1 + fundido 160 ms; el círculo nuevo hace un
  pequeño «pop» (1→1.08→1); queda 1.2 s; sale con fundido 200 ms. Un cambio
  nuevo mientras está visible lo reemplaza y reinicia el tiempo.
- No roba foco ni entrada (no reactivo).

Camino, igual que los menús:
1. tabletd: `shell::Feedback::ModeChanged { from, to }` → método D‑Bus nuevo
   `ShowModeOsd(s from, s to)` en la extensión (añadir al XML de
   `extension.js`, dibujar en `menus.js` o un `osd.js` nuevo, centrado en el
   monitor principal o en el del puntero).
2. Si la extensión no responde: línea JSON por stdin al `--shortcut-host`
   (`{"name":"mode_osd","from":"pen","to":"mouse"}`), parseada en
   `overlay::ShowRequest` (o un tipo hermano) y dibujada por egui con el mismo
   diseño, centrada; se cierra sola.
3. Tests: parseo de la línea, y captura `overlay-mode-osd.png`.
