# PLAN

Port de Tablet Center (Tauri + HTML/JS) a Rust puro con egui. Original en
`/home/projects/ia/wacom/app` (ver `AGENTS.md`). Referencias visuales en
`docs/reference/` (`classic-*` = interfaz «Tablet», `glass-*` = interfaz Glass).

## Estructura objetivo

```
crates/tablet-config   (copiado, sin cambios)
crates/tabletd         (copiado, sin cambios)
crates/tablet-center   app egui
  src/main.rs          arranque, instancia única, ventana
  src/daemon.rs        conexión persistente a tabletd (port de ask/exchange)
  src/system.rs        apps instaladas, autostart, backups, shell menus, ui style
  src/i18n.rs          locales + debrand
  src/theme.rs         tokens (clásica y Glass, oscuro y claro)
  src/assets.rs        iconos SVG, imágenes de dispositivos, fuentes
  src/widgets.rs       slider con marcas, switch, radio, select/menú, botón, chips…
  src/frame.rs         marco y barra de título propia
  src/classic/…        toolbar, sidebars y una página por archivo
  src/glass/…          interfaz Glass
  src/overlay/…        ventanas en pantalla (radial, rejillas, keypad) = shortcut.html
  tests/screenshots.rs capturas con egui_kittest
```

## Fases

1. **Base**: crate `tablet-center` egui que compila; tema, fuentes, iconos SVG,
   i18n, cliente del daemon, estado de la app (config/status con polling como
   app.js), datos simulados para capturas, marco + barra de título + toolbar
   (Devices / On-screen shortcuts / Settings) y la barra lateral de Devices.
   Arnés de capturas funcionando. Página Pen completa (ilustración del lápiz
   con etiquetas, Test area, Tip con sliders y ayudas «?», resto de tarjetas
   que aparecen en `classic-dark-devices-pen-tall.png`) y barra APPLICATIONS.
2. **Devices**: overview/welcome, ExpressKeys (tableta con líneas a las
   etiquetas), Mapping (monitores, trapecio, tableta, radios, switch, áreas
   parciales arrastrables), Display toggle, Device info, Touch, orientación,
   perfiles por aplicación (añadir/quitar app), menús de acción para botones
   (el selector de acciones del original con sus submenús y diálogos de tecla).
3. **On-screen shortcuts**: Start, Express menu, Radial menu (rueda dibujada
   igual), App shortcuts, Keypad, las seis páginas de gestos con su ilustración,
   Restore defaults. Ventanas en pantalla (overlay) equivalentes a
   `shortcut.html/js` y `run_shortcut_action`/`pending_shortcut` de main.rs.
4. **Settings**: General (servicio, Restart Driver, idioma, interfaz,
   autostart, menús de GNOME Shell), Devices manager, Backup & restore; toasts;
   diálogos de confirmación.
5. **Glass**: la interfaz alternativa completa (`glass-*`), con búsqueda,
   tarjetas y su barra de título; desenfoque de fondo en Wayland
   (ext-background-effect-v1, como `blur.rs` del original y de plane-llama).
   Cambiar de interfaz desde Settings sin reiniciar.
6. **Cierre**: instalador/paquete sin WebKitGTK (`install.sh`, `uninstall.sh`,
   `scripts/package.sh`), `.desktop`, README en inglés como el original con
   capturas nuevas generadas por el arnés y una sección «Development» que
   resuma las tareas de `just` (enlazando `docs/development.md`), revisión final de todas las capturas
   contra la referencia, clippy/tests en verde.
7. **Interfaz «Logi»** (eliminada después; ver `docs/progress.md`) (tercera interfaz, elegible en Settings → Interface junto a
   Tablet y Glass): misma funcionalidad, con el lenguaje visual de
   `docs/inspiration/openlogi/*.png` (OpenLogi): fondo casi negro (#1b1b1d),
   tarjetas #222225 con radio ~16, barra superior con «‹ Devices» + nombre del
   dispositivo + batería/«● Connected» en píldora; pantalla inicial de
   dispositivos con una tarjeta centrada (imagen del dispositivo, nombre, chip
   «Tablet»); barra lateral estrecha con iconos de línea (Pen, ExpressKeys,
   Mapping, Gestures, On-screen shortcuts, Device); fila de perfil
   «Profile [Default] … + Add app»; página de botones con la imagen grande del
   lápiz/tableta y tarjetas-callout «Etiqueta / ícono acción ›» unidas por
   líneas finas, más un panel lateral «Button inspector»; sliders azules
   (#2f5bea) con valor a la derecha, toggles «On/Off», controles segmentados;
   ventana de Settings con buscador y lista lateral como la última captura.
   Tipografía Noto Sans/Segoe-like, textos atenuados #9a9aa0.
