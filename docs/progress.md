# Progreso

## Fase 1 — Base nativa y página Pen

Implementación del 25 de octubre de 2026.

### Hecho

- Crate `tablet-center` con eframe/egui 0.36 y wgpu, Wayland y X11; sin Tauri ni WebView.
- Ventana sin decoraciones, título propio, botones nativos, arrastre, doble clic para maximizar y ocho zonas para redimensionar. Liberación del puntero después de los movimientos gestionados por el compositor.
- Tokens de colores, medidas y tipografía en `theme.rs`; SVG originales mediante `egui_extras`; imágenes originales del lápiz y de la tableta con filtrado previo al reducirlas.
- Roboto 3.015 con pesos 400/500/600/700 obtenido de los WOFF2 del original, en lugar del Roboto 2.137 que había en el repositorio. `tools/rebuild-fonts.py` permite reproducir la conversión y combinar latin, latin-ext y cyrillic. Fuentes embebidas y respaldo CJK del sistema cuando existe.
- Traducciones de `assets/locales`, diccionario inglés con las revisiones de `EXTRA_EN` y debrand clásico que conserva los nombres Wacom One.
- Cliente persistente de tabletd con JSON por líneas, tiempo límite de 500 ms, reconexión y ausencia de repetición tras un timeout. Polling de 40 ms y guardado/Reload en un hilo aparte; polling suspendido al minimizar.
- Datos simulados iguales al navegador: Intuos S 0x0374, dos monitores Dell, presión cero y `Config::default()`.
- Toolbar, navegación de Devices y barra APPLICATIONS con Other apps.
- Página Pen: ilustración, etiquetas y anillos, indicación de botones mantenidos, área de dibujo con presión y borrado tras dos segundos, tres sliders con puntos y teclado, ayudas que se abren al pasar el puntero o hacer clic, modos Pen/Mouse/Relative mouse y Restore defaults. Desplazamiento para acceder a las tarjetas inferiores en 720 px.
- Pruebas de transporte, timeout sin repetición, traducción/debrand y edición de controles/desplazamiento sin daemon ni escrituras en la configuración personal.
- En los crates copiados solo se añadieron tres anotaciones locales para avisos nuevos de clippy. `tablet-config` y el comportamiento de `tabletd` se conservan.

### Validación

- `cargo fmt --all`: ejecutado. Se conservaron los formatos originales de los crates copiados para evitar cambios ajenos a la fase; el crate nuevo queda formateado.
- `cargo clippy --workspace --all-targets -- -D warnings`: correcto.
- `cargo test --workspace`: 73 pruebas correctas; el test de capturas está ignorado por defecto.
- `cargo test -p tablet-center --test screenshots -- --ignored`: correcto, genera ambas capturas con wgpu a 1 pixel por punto.
- `tools/compare.sh classic-dark-devices-pen classic-dark-devices-pen-tall`: ejecutado y ambas comparaciones inspeccionadas visualmente varias veces.

| Captura | Tamaño | RMSE normalizado |
| --- | --- | --- |
| `classic-dark-devices-pen.png` | 1100 × 720 | 0.0221742 |
| `classic-dark-devices-pen-tall.png` | 1100 × 1600 | 0.0200742 |

Las capturas están en `target/screens` y las comparaciones con referencia a la izquierda, egui en el centro y diferencias a la derecha en `target/compare`.

### Diferencias visuales conocidas

La composición, los colores planos, el tamaño de las tarjetas y las posiciones principales están ajustados a la referencia. Aún hay diferencias finas en el rasterizado de Roboto, SVG, círculos, esquinas y reducción de las ilustraciones entre egui y el navegador. Las capturas no son idénticas al píxel; los RMSE anteriores documentan la diferencia restante. No se usan capturas de referencia como fondos ni como sustitutos de controles.

### Pendiente

- Alcanzar la igualdad visual estricta solicitada: todavía no se puede considerar cerrado ese criterio de aceptación.
- Fases 2–6 del plan: las demás páginas, perfiles por aplicación y selectores de acciones, shortcuts/overlays, Settings, Glass y empaquetado. La navegación reservada muestra el mensaje traducido del original para funciones futuras.
- Verificación manual de movimiento y redimensionado con el compositor de una sesión gráfica real. El arnés verifica la UI y las capturas, pero no simula al compositor.

Para abrir la UI con los datos simulados, sin daemon ni guardado de configuración:

```sh
cargo run -p tablet-center -- --simulate
```

### Commit bloqueado por el entorno

Se intentó crear `feat(fase-1): add native egui base and complete Pen page`, pero `.git` está montado como solo lectura. Tanto `git add` como `git commit` fallaron al crear `.git/index.lock`. Los cambios siguen en el árbol de trabajo, sin commit. No se modificaron permisos ni se intentó eludir esa restricción.

## Fase 2 — Devices

Implementación del 25 de octubre de 2026. Sin commit, según la indicación del supervisor.

### Hecho

- Overview con ilustraciones originales, brillo, enlaces de navegación y selector de las cuatro orientaciones con confirmación. Welcome cuando se desconecta el dispositivo.
- ExpressKeys con líneas, etiquetas y anillos interactivos; indicación de la tecla mantenida, acción seleccionada y restauración de los valores predeterminados del perfil.
- Mapping con distribución de monitores, selección desde el diagrama y los radios, trapecio, proporciones, orientación y áreas parciales de pantalla/tableta. Arrastre del área y ocho tiradores; límites normalizados y tamaño mínimo del 5 %. Restaurar Mapping conserva la orientación.
- Display toggle con ilustración, animación en modo nativo e indicación del monitor activo informado por el daemon. Captura determinista en modo simulado.
- Device info con identificadores, firmware y diagnóstico vivo de inclinación/presión. Touch con activación persistente y navegación cuando el dispositivo anuncia esta capacidad.
- Barra de aplicaciones compartida por Pen y ExpressKeys; selección de perfil, desplazamiento, añadir y eliminar con confirmación. Añadir copia los ajustes del perfil seleccionado; eliminar conserva los ajustes globales.
- Selector de aplicaciones instaladas/en ejecución, búsqueda, selección por teclado, doble clic, Enter y Escape. Descubrimiento de archivos `.desktop` y consulta `RunningApps` en un hilo aparte; se respetan XDG, Hidden y NoDisplay.
- Menús de acciones para los tres controles del lápiz y las ExpressKeys: grupos desplegables, iconos originales, presets, captura de combinación física a nombres evdev, modificadores/clic, comando/aplicación, precisión, cambio de modo y tiempo de pulsación para Right · hold Middle. Los cambios usan el guardado asíncrono existente.
- Medidas y colores de Devices centralizados en `theme.rs`. Los crates `tablet-config` y `tabletd` no cambian en esta fase.
- Arnés ampliado a las catorce referencias oscuras de Devices a 1100×720 y 1100×1600. El propio test rechaza un RMSE RGB normalizado superior a 0,03.
- Cuatro capturas extra sin referencia: `classic-dark-actions-open`, `classic-dark-add-application`, `classic-dark-add-application-list` y `classic-dark-keystroke-capture`. Revisadas frente a las medidas y controles de `style.css`/`select.css`.

### Validación

- `cargo fmt --all`: ejecutado; se conserva el formato previo de los crates copiados, como en la fase 1.
- `cargo clippy --workspace --all-targets -- -D warnings`: correcto.
- `cargo test --workspace`: 81 pruebas correctas; un test de capturas ignorado por defecto.
- `cargo test -p tablet-center --test screenshots -- --ignored`: correcto, con wgpu y un píxel por punto.
- `tools/compare.sh --all`: ejecutado. Las catorce imágenes de comparación se inspeccionaron visualmente y se corrigieron en varias iteraciones.

| Captura (sin extensión) | 1100 × 720 | 1100 × 1600 (`-tall`) |
| --- | --- | --- |
| `classic-dark-devices` | 0.0240196 | 0.0165671 |
| `classic-dark-devices-expresskeys` | 0.0274806 | 0.0221320 |
| `classic-dark-devices-mapping` | 0.0289097 | 0.0200912 |
| `classic-dark-devices-display-toggle` | 0.0293820 | 0.0205119 |
| `classic-dark-devices-info` | 0.0168529 | 0.0118060 |
| `classic-dark-devices-touch` | 0.0190139 | 0.0132008 |
| `classic-dark-devices-pen` | 0.0253771 | 0.0217143 |

RMSE calculado sobre RGB; el máximo es 0,0293820.


### Diferencias visuales conocidas y alcance de la verificación

El criterio de aceptación de esta fase es RMSE ≤ 0,03 por referencia, según la nota del supervisor. Todas las capturas lo cumplen. Persisten diferencias de rasterizado de Roboto/SVG, curvas, trazos y brillo entre navegador y egui. No se utilizan referencias como fondos ni para sustituir controles.

Las pruebas de interacción cubren áreas parciales, perfiles aislados, captura física de teclas, modificación de botones del lápiz, diálogos parametrizados, Touch, restauración de Mapping y confirmación de orientación. Usan datos simulados y no escriben en la configuración personal. No se ha instalado ni reiniciado tabletd; queda para una sesión gráfica real comprobar el resultado con hardware y compositor.

La igualdad estricta al píxel pendiente en la fase 1 queda sustituida, para esta entrega, por el criterio visual y RMSE aprobado por el supervisor. Las fases 3–7 quedan fuera de esta implementación.

## Fase 3 — On-screen shortcuts y ventanas en pantalla

Implementación del 25 de octubre de 2026. Sin commit, por indicación del supervisor.

### Hecho

- Start y navegación completa, con sidebar desplazable, iconos originales y texto Legacy de dos líneas.
- Express menu y Radial menu: ocho sectores SVG, hub, sombras, separadores, estados hover/selección, iconos y etiquetas. App shortcuts y Keypad: rejillas de cuatro columnas, cabecera y tecla «0» de dos columnas. Retroceso vectorial para evitar un glifo ausente.
- Edición de acciones, nombre personalizado y captura física de teclas mediante el selector compartido con Devices. Los menús excluyen los gestos que requieren mantener un botón. Tamaño, texto, «Display at cursor» y Restore defaults por menú; guardado asíncrono.
- Las seis páginas de gestos, ilustraciones SVG originales, traducciones, velocidades independientes y «Content follows the pen».
- Corrección de fase 2: Actions sin «×», cierre por Escape/clic fuera y barra fina desplazable con rueda/arrastre cuando hay desbordamiento.
- Host persistente compatible con stdin de tabletd: petición pendiente, alternancia, sustitución, configuración leída en otro hilo y salida al terminar stdin.
- Viewport egui diferido, transparente, maximizado, sin decoraciones, siempre encima y sin entrada en la barra de tareas. Monitor del lápiz, compensación de paneles, límites de ocho píxeles, centrado para ExpressKeys y «Display at cursor». App::logic despierta el host oculto mediante una ventana transparente de anclaje de un píxel.
- Animación con escala/opacidad; cierre por hub/cabecera, clic fuera, Escape o pérdida de foco. Disabled conserva el menú abierto. run_shortcut_action oculta y espera 150 ms antes de enviar Request::Run en otro hilo.
- Renderizado compartido entre configuración y overlays. Los crates copiados permanecen sin cambios.

### Validación

- cargo fmt --all ejecutado; se conserva el formato previo de los crates copiados.
- cargo clippy --workspace --all-targets -- -D warnings: correcto.
- cargo test --workspace: 87 pruebas correctas; tres pruebas de capturas ignoradas.
- Capturas ignoradas: correctas, wgpu a un píxel por punto; inicialización de GPU serializada para evitar fallos del controlador al ejecutar varios tests; 22 referencias de Shortcuts y 14 de Devices, todas con RMSE ≤ 0,03.
- tools/compare.sh --all: ejecutado; inspección individual de las 22 comparaciones de esta fase e iteraciones de corrección.
- Capturas propias revisadas contra style.css/shortcut.js: overlay-express-menu, overlay-keypad, overlay-app-shortcuts y classic-dark-shortcuts-actions-open. Los PNG de overlays conservan transparencia.
- Ejecutable probado con tools/smoke-shortcuts.py en Xvfb aislado: abrir Keypad, alternar cierre, reabrir App shortcuts, pulsar Undo y verificar Ctrl+Z mediante Run en un socket simulado, comprobar ventana oculta, Escape y salida limpia al terminar stdin. Configuración/socket en target/native-smoke; sin daemon real.

| Captura (prefijo classic-dark-shortcuts) | 720 | 1600 |
| --- | --- | --- |
| Start | 0.0298787 | 0.0204483 |
| -express-menu | 0.0279299 | 0.0191656 |
| -radial-menu | 0.0264069 | 0.0181669 |
| -app-shortcuts | 0.0271665 | 0.0186731 |
| -keypad | 0.0256724 | 0.0189084 |
| -pan-scroll | 0.0227483 | 0.0157867 |
| -pan-scroll-legacy | 0.0217269 | 0.0151254 |
| -pan-zoom | 0.0236515 | 0.0163732 |
| -scroll | 0.0197088 | 0.0138274 |
| -wheel | 0.0192283 | 0.0135178 |
| -zoom | 0.0177761 | 0.0125914 |

Máximo: 0,0298787. Devices mantiene los valores de la fase 2.

### Diferencias visuales conocidas y verificación pendiente

Persisten diferencias finas de rasterizado de fuentes/SVG, curvas y sombras. No se persigue antialiasing subpíxel ni se usan referencias como fondos.

X11 valida el ciclo nativo y el protocolo. Falta comprobar con tableta/compositor Wayland reales monitor, foco, transparencia compuesta y efecto sobre otra aplicación, especialmente con escalas distintas. No se instaló ni reinició tabletd ni se escribió en la configuración personal.

Fase 3 implementada. Fases 4–7 fuera de alcance; el supervisor hará el commit.


## Fase 4 — Settings, backups y mensajes

Implementación del 25 de octubre de 2026. Sin commit, por indicación del supervisor.

### Hecho

- General completo: estado del servicio, Restart Driver, los catorce idiomas, selector Tablet/Glass, autostart y menús de GNOME Shell. Desplazamiento a 720 px con coordenadas compartidas por dibujo, puntero y accesibilidad.
- Cambio de idioma inmediato en toda la UI, incluidos toolbar, navegación, acciones y nombres de interfaz. Comprobado con español y japonés. Fuente CJK del sistema y ajuste de su línea base; botones y toolbar miden sus textos traducidos. La preferencia nativa de idioma queda en el archivo `language` junto a `config.json`, sustituyendo el almacenamiento local del WebView.
- Device manager con nombre, modelo e ilustración del dispositivo conectado; modelos e ilustración genérica equivalentes a `MODELS`/`tabletSource` del original y estado vacío al desconectarlo.
- Backups: listado ordenado por fecha, creación con nombre de 1–120 caracteres, restauración, borrado, apertura de carpeta y reset global. Nombres largos se ajustan a la columna; las acciones se dimensionan según la traducción. Crear guarda primero la configuración actual. Restaurar/reset actualizan el estado visible y descartan perfiles seleccionados que ya no existen.
- Port de `restart_daemon`, `autostart_enabled`, `set_autostart`, `list_backups`, `create_backup`, `restore_backup`, `delete_backup`, `open_backups_folder`, `reset_config`, `save_shell_menus` y `set_ui_style`. Operaciones de archivos, procesos y daemon en hilos de trabajo.
- Compatibilidad de rutas: configuración, `interface` y `shell-menus.json` respetan `XDG_CONFIG_HOME` como el original; backups en `$HOME/.config/tablet-center/backups`; autostart en `$HOME/.config/autostart/tablet-center.desktop`, con Exec a `$HOME/.local/bin/tablet-center` y el mismo escapado. Backups con el mismo JSON `{title, config}`, nombres y rechazo de rutas/symlinks.
- Elección de Glass persistida en `interface`, recuperada al iniciar. La pantalla sigue siendo clásica hasta implementar Glass en la fase 5.
- Exportación de los cuatro menús de GNOME con títulos/etiquetas traducidos, iconos simbólicos, acciones, tamaños y span de la tecla 0. Se actualizan al editar la configuración o cambiar idioma.
- Guardados atómicos y protección de los cambios restaurados/reset frente a guardados anteriores aún en cola. Toast traducido cuando el archivo se guarda sin un daemon disponible.
- Selectores con lista desplazable, marca SVG de `select.css`, halo, selección por teclado, Home/End, Enter, Escape y cierre exterior. Toast de 3500 ms. Diálogos de crear, restaurar, borrar y reset con fondo modal, botón primario, foco inicial, ciclo de Tab, Escape y devolución de foco.
- Tokens de Settings en `theme.rs`; SVG de la tableta genérica y marca de selección derivados del original. Los crates copiados permanecen sin cambios.

### Correcciones del supervisor

- Callouts de Pen y ExpressKeys en una columna a la derecha del dibujo (orden sin cruces de líneas); barra lateral marca la página activa; inspector con lista en `ScrollArea` dentro del panel; tarjetas con alto según contenido (Mapping, Shortcuts, Device, Display toggle); fila inferior de Actions Ring en dos líneas; «Device details»; «Manage this device» refleja el servicio; perfil por defecto «Other apps»; botones Settings y «+ Add app» funcionan; iconos de Logi ×1,25.
- Todas las interfaces: esquinas de la ventana transparentes también bajo barras laterales, paneles y velos de diálogo (`frame::round_window`); brillo tenue bajo las imágenes oscuras del lápiz y la tableta en modo oscuro (`assets::glow`).
- Lápiz en Wayland: el trazo del área de prueba usa todas las muestras del frame y uniones redondas (antes se veían «palitos»), y arrastrar o redimensionar la ventana con el lápiz llama a `xdg_toplevel.move/resize` con el serial del toque.

### Validación

- `just fix`: correcto. Las tareas de formato se limitan a `tablet-center` para conservar intactos los crates copiados; clippy y tests siguen cubriendo todo el workspace. Alcance documentado en `docs/development.md`.
- `just check`: correcto; formato, clippy con `-D warnings` y 92 pruebas del workspace. Cuatro pruebas de capturas ignoradas por defecto.
- `just screens`: correcto; las cuatro pruebas de capturas pasan con wgpu y un píxel por punto. Las 42 referencias oscuras de Devices, Shortcuts y Settings cumplen RMSE ≤ 0,03.
- `just reference-settings`: correcto. Genera referencias adicionales desde el HTML/CSS original sin modificarlo, en `target/settings-reference`; Chromium usa un perfil en `target`.
- `just compare` para las seis referencias de Settings y `REFERENCE_DIR=target/settings-reference just compare …` para los diez estados adicionales: ejecutados. Las dieciséis comparaciones se inspeccionaron individualmente con la herramienta de imágenes, con varias iteraciones de ajuste.
- Pruebas de interacción: idioma con ratón y teclado, navegación traducida, español/japonés, interfaz, switches, cancelación, reset y desplazamiento de General a 720 px.
- Pruebas de archivos/comandos en un subproceso con HOME/XDG temporales y `systemctl`/`xdg-open` simulados: autostart, escapado, interfaz, esquema de backups, validación, symlinks, errores y exportación GNOME. Recorrido de UI crear → reset → restaurar → borrar con archivos reales temporales.

RMSE RGB normalizado; valores completos en `target/settings-rmse.json`.

| Captura (prefijo `classic-dark-settings-`) | 720 | 1600 (`-tall`) |
| --- | --- | --- |
| `general` | 0.0198543 | 0.0146568 |
| `devices` | 0.0167276 | 0.0117697 |
| `backups` | 0.0147050 | 0.0104842 |
| `general-es` | 0.0217838 | 0.0159285 |
| `general-ja` | 0.0163319 | 0.0171640 |

| Estado adicional (1100 × 720) | RMSE |
| --- | --- |
| `language-open` | 0.0236791 |
| `interface-open` | 0.0204602 |
| `toast` | 0.0231842 |
| `reset-confirmation` | 0.0198851 |
| `delete-confirmation` | 0.0204742 |
| `create-dialog` | 0.0163576 |

Máximo de las dieciséis capturas de esta fase: 0.0236791. Capturas en `target/screens`; comparaciones en `target/compare`, con referencia a la izquierda, egui en el centro y diferencias a la derecha.

### Diferencias visuales conocidas y alcance pendiente

La composición, las medidas principales, los textos y los controles se cotejaron contra las referencias y `style.css`/`select.css`. Persisten diferencias finas de rasterizado de fuentes/SVG, bordes y difusión de sombras entre Chromium y egui. No se usan referencias como fondos ni para sustituir controles.

No se instaló ni reinició el servicio real ni se escribió en la configuración personal. La ejecución efectiva de Restart Driver, autostart y apertura de carpeta se valida con herramientas simuladas; su integración con la sesión gráfica real queda para la comprobación manual. También queda comprobar la extensión de GNOME en una sesión real.

Fase 4 implementada y validada con el criterio RMSE solicitado. Fases 5–7 fuera de este trabajo. El supervisor hará el commit.


## Fase 5 — Glass

Implementación del 25 de octubre de 2026. Sin commit, por indicación del supervisor.

### Hecho

- Presentación Glass nativa, sin WebView: navegación de Pen, ExpressKeys, Mapping, Gestures, On-screen shortcuts, Device info y Settings. Tarjetas, ilustraciones, controles, barra de título y botones SVG derivados de `glass.html`, `glass.css`, `glass.js` y `frame.css`.
- Configuración, perfiles por aplicación, estado del daemon, guardado asíncrono y operaciones de Settings compartidos con Tablet. Cambio de interfaz inmediato; la preferencia `interface` se recupera al arrancar. Volver a Tablet desde Settings mantiene esa página.
- Noto Sans embebida, con Regular, Medium y Bold e hinting. Títulos de 29/17 px y tracking de −0,9/−0,25 px; etiquetas y controles según CSS. Roboto conserva su configuración anterior. Colores, medidas y radios de Glass centralizados en `theme.rs`.
- Glass sigue el tema claro/oscuro del sistema y conserva el último tema conocido entre cuadros. Tablet mantiene su paleta clásica. Traducciones compartidas; el debrand se aplica solo a Tablet.
- Pen con acciones, tiempo de pulsación condicional, presión, sensibilidad, distancia de doble clic (incluidos 6 mm), prueba de dibujo, borrado y modo ratón. ExpressKeys con identificación de botones y edición aislada del perfil seleccionado. Mapping con monitores seleccionables, All displays, proporciones y destino vivo de Display toggle.
- Las seis tarjetas de gestos editan los campos compartidos. Los cuatro menús permiten cambiar tamaño, texto, posición, etiquetas y acciones; rejillas con tecla 0 de dos columnas. Acciones con filtro, catálogo compartido, presets, captura física de teclas, modificadores y comandos. Toast con Undo para cambios de configuración.
- Settings con servicio, idioma, interfaz, autostart, menús de GNOME y backups; presentación de diálogos Glass sobre las operaciones de archivos de la fase 4.
- Search settings con el catálogo y las rutas de `glassSearch`: filtro traducido, doce resultados como máximo, desplazamiento, entrada por teclado, navegación, cierre exterior y Escape. Capturas propias del desplegable en ambos temas.
- `blur.rs` enlaza `ext-background-effect-v1` sobre la conexión y la superficie Wayland de winit. Región redondeada, cambios de tamaño y capacidad, activación solo para Glass y retirada al volver a Tablet. Sin protocolo/capacidad, el fondo queda opaco. El efecto se destruye antes que la ventana.
- Nuevas tareas `just screens-glass` y `just reference-glass`, documentadas en `docs/development.md`. Chromium usa perfiles en `target`.

### Referencias corregidas

El generador anterior abría `glass.html#/expresskeys`; esa ruta no existe en `glass.js` y mostraba Pen. Se corrigió a `#/keys` y se regeneraron únicamente las cuatro referencias de ExpressKeys desde el HTML original, sin modificarlo. Las otras 24 referencias de páginas coinciden exactamente con las capturas nuevas del original y se conservaron. Se añadieron las dos referencias de búsqueda con la consulta `pen`.

### Correcciones del supervisor

- Callouts de Pen y ExpressKeys en una columna a la derecha del dibujo (orden sin cruces de líneas); barra lateral marca la página activa; inspector con lista en `ScrollArea` dentro del panel; tarjetas con alto según contenido (Mapping, Shortcuts, Device, Display toggle); fila inferior de Actions Ring en dos líneas; «Device details»; «Manage this device» refleja el servicio; perfil por defecto «Other apps»; botones Settings y «+ Add app» funcionan; iconos de Logi ×1,25.
- Todas las interfaces: esquinas de la ventana transparentes también bajo barras laterales, paneles y velos de diálogo (`frame::round_window`); brillo tenue bajo las imágenes oscuras del lápiz y la tableta en modo oscuro (`assets::glow`).
- Lápiz en Wayland: el trazo del área de prueba usa todas las muestras del frame y uniones redondas (antes se veían «palitos»), y arrastrar o redimensionar la ventana con el lápiz llama a `xdg_toplevel.move/resize` con el serial del toque.

### Validación

- `just fix`: correcto.
- `just check`: correcto; formato, clippy con `-D warnings` y 97 pruebas del workspace. Cinco pruebas de capturas ignoradas por defecto.
- `just screens`: correcto; cinco pruebas con wgpu, 1100 × 720 y 1100 × 1600, un píxel por punto. Las 28 referencias Glass y los dos desplegables de búsqueda cumplen RMSE RGB normalizado ≤ 0,03. Las capturas clásicas de las fases anteriores también pasan.
- `just reference-glass`: ejecutado; genera referencias adicionales y geometría CSS en `target/glass-reference`. No escribe en el original.
- `just compare` para las 28 páginas y ambos desplegables: ejecutado. Se inspeccionaron individualmente las treinta comparaciones y se iteró sobre fuentes, ilustraciones, geometría de tarjetas, monitores, controles y marco.
- Pruebas de interacción: búsqueda y navegación, cambio Tablet/Glass sin perder configuración, aislamiento del perfil de aplicación, acciones del lápiz/ExpressKeys, Undo, tema del sistema y conservación del tema clásico. Prueba de la región redondeada de blur. Las pruebas de preferencias y archivos de la fase 4 siguen pasando con directorios temporales.

| Captura (`glass-*`) | Oscuro 720 | Oscuro 1600 | Claro 720 | Claro 1600 |
| --- | --- | --- | --- | --- |
| `pen` | 0.0236462 | 0.0241497 | 0.0250743 | 0.0259164 |
| `expresskeys` | 0.0175091 | 0.0163236 | 0.0197033 | 0.0181231 |
| `mapping` | 0.0255795 | 0.0230605 | 0.0278536 | 0.0251150 |
| `gestures` | 0.0182499 | 0.0163379 | 0.0197173 | 0.0170417 |
| `shortcuts` | 0.0237036 | 0.0248466 | 0.0255323 | 0.0274940 |
| `info` | 0.0209562 | 0.0141019 | 0.0219997 | 0.0147655 |
| `settings` | 0.0160525 | 0.0109083 | 0.0174981 | 0.0118234 |

Búsqueda: oscuro 0.0267142; claro 0.0280907. Máximo de las treinta capturas: 0.0280907. Valores reproducibles en `target/glass-rmse.json`; capturas en `target/screens` y comparaciones en `target/compare`.

El arnés compone las esquinas transparentes sobre el mismo fondo que Chromium; la ventana nativa conserva la transparencia. No se usan referencias como fondos ni para sustituir controles.

### Diferencias conocidas y comprobación nativa pendiente

Persisten diferencias finas de rasterizado de Noto Sans/SVG, bordes, círculos y sombras entre Chromium y egui. La composición, las medidas, las paletas, los textos y los iconos de las referencias se cotejaron visualmente y cumplen el criterio RMSE.

Queda comprobar el efecto de desenfoque con un compositor Wayland real que anuncie el protocolo, incluido el cambio de interfaz y escalas distintas. El arnés valida la presentación opaca y la región del protocolo; no valida la imagen del escritorio desenfocada. También permanece la comprobación con hardware real ya señalada en las fases anteriores. No se instaló ni reinició tabletd ni se escribió en la configuración personal.

Fase 5 implementada y validada con las limitaciones nativas anteriores. Fases 6 y 7 fuera de este trabajo. El supervisor hará el commit.

## Fase 5b — hover y lápiz

- Estados `:hover`/`:active`/`:focus-visible` del CSS original en Tablet y Glass, popup «?» al pasar el puntero, cursor de mano; test de píxeles en `tests/interaction.rs`.
- Entrada del lápiz en Wayland (`tablet_input.rs`, protocolo tablet-v2): winit 0.30 no lo implementa y GNOME no emula el puntero. Probado a mano con la Intuos S, igual que el blur de Glass.
- Pendiente: el lápiz no llega a las ventanas en pantalla (overlays), solo a la ventana principal.

## Fase 6 — Cierre: instalador, paquete y README

Implementación del 25 de octubre de 2026. Sin commit, por indicación del supervisor.

### Hecho

- `install.sh` sin la parte WebKitGTK: el único tramo con privilegios es la regla udev de `uinput` y el grupo `input`. El resto es el del original: binarios de `bin/` (archivo de release) o build con cargo, espera del lápiz lejos de la tableta, servicio systemd de usuario con restart, `.desktop` con `Exec` reescrito al `$HOME`, iconos 48–512, extensión de GNOME habilitada con gsettings y mensajes finales según el estado.
- `uninstall.sh` y `scripts/package.sh` del original, sin cambios: ya no referencian WebKitGTK ni Tauri. `package.sh` construye con `cargo build --locked --release` y empaqueta `bin/`, `packaging/`, `gnome-extension/`, `install.sh`, `uninstall.sh`, `LICENSE` y `README.md` en `dist/tablet-center-<arch>-linux.tar.gz`. `--purge` sigue borrando además los datos `dev.tabletcenter.linux` que dejaba la versión Tauri.
- `packaging/tablet-center.desktop` ya traía `StartupWMClass=tablet-center` e `Icon=tablet-center`; la app usa `with_app_id("tablet-center")` (app_id de Wayland) y el binario se llama `tablet-center` (WM_CLASS en X11), así que el `.desktop` casa con la ventana.
- `README.md` nuevo en inglés, con la estructura del original (intro, Install, Screenshots, Features, Details, From source, How it works, Development, Credits, License), sin WebKitGTK/Tauri/bun: Development resume las tareas de `just` y enlaza `docs/development.md`; How it works describe la app egui nativa. Las capturas son las del arnés egui, copiadas a `docs/screenshots/` con los nombres del original (`pen-settings`, `expresskeys-settings`, `express-menu-settings`, `pan-scroll-settings`, `mapping-settings`, `glass-dark`).

### Validación

- `sh -n` en `install.sh`, `uninstall.sh` y `scripts/package.sh`: correcto.
- Prueba aislada de los instaladores (no se ejecutaron de verdad): copias de los scripts en `/tmp/opencode/phase6`, `HOME` temporal, `env -i`, binarios stub de tabletd/tablet-center sin la marca del lápiz y `systemctl`/`gsettings`/`sudo`/`usermod`/`udevadm` stub que registran las llamadas. Instalación completa verificada archivo a archivo (binarios, servicio, `.desktop` con `Exec` al HOME temporal, cinco iconos, cuatro archivos de la extensión) y en el log (`daemon-reload`, `enable`, `restart`, `gsettings set` con la extensión). Rama con privilegios verificada aparte con la ruta de la regla udev redirigida y un `id` sin grupo input: `sudo install` de la regla, `udevadm control --reload` + `trigger` y `usermod -aG input`. `uninstall.sh --purge` elimina binarios, servicio, `.desktop`, autostart, iconos, extensión, `~/.config/tablet-center` y los datos del Tauri antiguo, y quita la extensión de `enabled-extensions`; solo queda `mimeinfo.cache` regenerado. Comprobado después que el tabletd, la regla udev, el servicio y la extensión reales siguen intactos.
- `just build`: correcto. `just package`: correcto, `dist/tablet-center-x86_64-linux.tar.gz` con el `install.sh` nuevo y el README.
- `just fix` y `just check`: correctos; clippy con `-D warnings` y 99 pruebas del workspace (cinco de capturas ignoradas por defecto).
- `just review`: correcto; las 72 comparaciones con referencia cumplen RMSE RGB ≤ 0,03.

| Revisión final | RMSE |
| --- | --- |
| Máximo (72 capturas) | 0,0298760 (`classic-dark-shortcuts`) |
| Siguiente | 0,0293805 (`classic-dark-devices-display-toggle`) |

### Pendiente

- Fase 7 (interfaz «Logi») y las comprobaciones con sesión gráfica real ya anotadas en fases anteriores.
- Instalación real en una máquina limpia y una actualización desde la versión Tauri; aquí solo se probó en sandbox porque la máquina tiene el servicio de producción.

## Fase 7 — Interfaz «Logi»

Implementación del 25 de octubre de 2026 (GLM-5.3; correcciones finales del supervisor).

### Hecho

- Tercera interfaz «Logi» nativa según `docs/inspiration/openlogi/SPEC.md`: fondo `#1a1a1d`, tarjetas `#222227` radio 14, barra de título de 22 px con «Tablet Center» centrado y botones ─ ☐ ✕ de línea (✕ rojo al pasar el puntero), barra superior de 57 px con «‹ Devices» + nombre del dispositivo + píldora «● Connected» (gris «Disconnected» sin daemon), barra lateral de 145 px con iconos de línea (Pen, Pen feel, ExpressKeys, Mapping, Actions Ring, Shortcuts, Device; seleccionado `#27344c`), fila de perfil con chips «Default»/aplicaciones y «+ Add app», misma tipografía Noto Sans que Glass y textos atenuados `#9a9aa0`.
- Pantalla inicial «Devices» con «N device» y botón «⚙ Settings»; tarjeta centrada 334×240 con icono USB verde, imagen de la tableta, nombre, chip «Tablet» y «⋯»; clic abre el dispositivo y «‹ Devices» vuelve.
- Pen y ExpressKeys con la ilustración grande del lápiz/tableta, callouts 140×50 («etiqueta / ícono acción ›») unidos por línea horizontal + diagonal con círculo hueco de 7 px, borde `#3b82f6` al seleccionar o mantener pulsado, y panel derecho «Button inspector» de 290 px con la lista completa de acciones del clásico (recomendados específicos del lápiz, captura física de teclas, modificador y ejecutar aplicación), fila actual sobre `NAV_SEL` y slider «Hold time» para Right · hold Middle.
- Pen feel en rejilla de dos columnas: Tip feel, Pressure to click, Double-click distance (con paso de 6 mm) y Pen mode; sliders azul `#1d4ed8` con riel `#212b4a`, perilla negra y valor en azul a la derecha, toggles 26×15 con «On/Off», control segmentado de tres modos y Restore defaults.
- Mapping con la tarjeta «Screen area» recoloreada (monitores Dell seleccionables, All displays, tableta, orientación segmentada, Force proportions, áreas parciales conmutables), y tarjeta «Display toggle» con destino vivo. Actions Ring con rueda de 240 px, ocho botones de 42 px, aro del slot activo, tarjeta de 280 px con «Clear slot» y selector de acción, y fila «Open at the current cursor position.». Shortcuts con tarjetas por menú (chips editables con diálogo de nombre+acción, Size, Text size, Display at cursor, Restore defaults) y las seis tarjetas de gestos con Speed y «Content follows the pen». Device con columna de 560 px: detalles (Connection/Model/Product ID/Firmware), Display toggle, Touch si aplica, y Configuration («Manage this device» que arranca o detiene el driver, Active profile, Button bindings, «⚙ Settings» y «🗀 Config folder»).
- Ventana de Settings estilo OpenLogi: columna izquierda `#0a0a0a` de 250 px con buscador y secciones (General, Devices, Backups, Appearance, About; seleccionado `#2b2b2d`), encabezado de 52 px, tarjeta única con filas y descripciones; Appearance ofrece Tablet/Glass/Logi (cambio en vivo, sin reinicio, persistido en `interface` como «logi») y los 14 idiomas; «‹ Back» o Esc vuelve. Comparte el estado de Settings/daemon/backups/toasts y diálogos (crear/restaurar/borrar/reset) presentados en Logi.
- Integración: `App` enruta `interface == "logi"`; aislamiento de perfiles por aplicación igual que Glass (clonación al entrar, restauración al salir); guardado asíncrono y shell menus sin cambios; debrand solo en la clásica; helpers de `glass.rs` (`get`/`set`, `apply_action`, `tablet_art`, `key_spot`, `device_name`) y de `devices.rs` (`poll_apps`, `submit_app`, catálogo de acciones) reutilizados como `pub(crate)`; `frame::resize` extraído para los bordes redimensionables; `system::set_ui_style` acepta «logi» y añade `stop_daemon`.
- Tokens nuevos en `theme::logi` (paleta y medidas del SPEC); sin colores literales en la UI. Hover con cursor de mano en todos los interactivos. Textos nuevos añadidos a los 14 locales (inglés en los no traducidos).
- Pruebas: `logi_interface_home_and_back_navigation` (elegir «Logi» en Settings cambia la interfaz, «‹ Back» cierra Settings, «‹ Devices» vuelve al inicio y la tarjeta abre el dispositivo) y capturas `logi-dark-{home,pen,pen-selected,pen-feel,expresskeys,mapping,actions-ring,shortcuts,device,settings}.png` a 1100×720 con los mismos datos simulados, generadas por `just screens` y sin referencia (revisión visual del supervisor).

### Correcciones del supervisor

- Callouts de Pen y ExpressKeys en una columna a la derecha del dibujo (orden sin cruces de líneas); barra lateral marca la página activa; inspector con lista en `ScrollArea` dentro del panel; tarjetas con alto según contenido (Mapping, Shortcuts, Device, Display toggle); fila inferior de Actions Ring en dos líneas; «Device details»; «Manage this device» refleja el servicio; perfil por defecto «Other apps»; botones Settings y «+ Add app» funcionan; iconos de Logi ×1,25.
- Todas las interfaces: esquinas de la ventana transparentes también bajo barras laterales, paneles y velos de diálogo (`frame::round_window`); brillo tenue bajo las imágenes oscuras del lápiz y la tableta en modo oscuro (`assets::glow`).
- Lápiz en Wayland: el trazo del área de prueba usa todas las muestras del frame y uniones redondas (antes se veían «palitos»), y arrastrar o redimensionar la ventana con el lápiz llama a `xdg_toplevel.move/resize` con el serial del toque.

### Validación

- `just fix`: correcto. `just check`: correcto (clippy `-D warnings`, 100 pruebas del workspace, seis de capturas ignoradas por defecto).
- `just screens`: correcto. `just review`: las 72 capturas con referencia cumplen RMSE ≤ 0,03 (máximo 0,0298760, sin cambios respecto a la fase 6).
- Capturas Logi verificadas por muestreo de píxeles contra los tokens del SPEC (título, barra lateral, nav seleccionado, tarjetas, inspector, slider, rueda, píldora verde, columna de Settings), además de la prueba de interacción.

### Pendiente

- Revisión visual fina de las diez capturas `logi-dark-*` por parte del supervisor (no hay referencia al píxel posible: el original OpenLogi es otra aplicación).
- El selector clásico Tablet/Glass añadió la fila «Logi»: el estado adicional `classic-dark-settings-interface-open` (referencias extra de la fase 4, fuera de `just review`) pasa de RMSE 0,0205 a 0,0406; diferencia intencional y documentada.
- Áreas parciales de Mapping se activan por segmentos (rectángulo 80 %) sin arrastre de tiradores, y la orientación se aplica sin diálogo de confirmación; edición fina disponible en la interfaz clásica.
- Traducciones Logi pendientes en los 13 idiomas distintos del inglés (actualmente en inglés) y comprobación con tableta/compositor reales, como en fases anteriores.

### Sliders estilo Glass en la interfaz Tablet

- Los sliders de la clásica usan la perilla de Glass (aro y punto `ACCENT`
  sobre el riel, sin relleno azul ni perilla blanca). Es un cambio pedido a
  propósito, así que Pen y las páginas de atajos con sliders quedan con RMSE
  hasta 0,037 y su umbral pasa a 0,04.
- Glass: «Try your pen» y el porcentaje ya no se encimen en idiomas con textos
  más largos (el porcentaje va tras el ancho medido de la etiqueta).

### Menos VRAM, iconos Lucide en Tablet y fin de Glass

- VRAM: eframe crea el dispositivo wgpu con `MemoryHints::Performance`, que
  reserva ~200 MiB por adelantado en NVIDIA. `main.rs` pasa a
  `MemoryHints::MemoryUsage`: en simulación, 216 → 35–47 MiB (`nvidia-smi`),
  similar a plane-llama-rust con OpenGL (43 MiB).
- La interfaz Tablet usa los iconos Lucide de Logi (`src/icons.rs`, compartido):
  navegación, barra lateral, catálogo de acciones y atajos. Se añadieron
  `hand.svg` (Touch) y `house.svg` (Start) y se borraron los SVG clásicos sin uso.
  Las «×» de cerrar los menús usan `x`.
- Glass eliminado: `glass.rs`, `blur.rs` (solo lo usaba Glass), `theme::glass`,
  sus pruebas y capturas, `docs/reference/glass-*`, `tools/capture-glass-reference.mjs`,
  las tareas `screens-glass`/`reference-glass` y la clave «Glass» de los locales.
  Los helpers que Logi compartía pasan a `src/shared.rs`. Un archivo
  `interface` con «glass» abre la clásica.
- Umbrales de RMSE de las capturas: 0,05 en general y 0,06 en atajos, porque los
  iconos ya no son los del original (máximo 0,058, `classic-dark-shortcuts-app-shortcuts`).
- Las ranuras desactivadas del menú radial muestran «ban» (antes una «×»).

### Ventana, textos en español, menú de acciones y bordes

- Ventana inicial de 1100×880 (antes 720), suficiente para la tableta, el lápiz
  y las ExpressKeys sin cortar; en el primer frame con monitor conocido se
  limita al 90 % de la pantalla. Las capturas siguen a 1100×720.
- Textos largos (revisado en español con las nuevas capturas `classic-dark-es-*`):
  las etiquetas de las barras laterales que no caben hasta el borde se parten en
  dos líneas algo más chicas (`widgets::fit_text`); los botones sobre la tableta
  y «Lápiz» centran icono+texto y se ensanchan; la píldora de aplicación y el
  estado de la derecha miden su texto; el subrayado de «Información del
  dispositivo» y el «?» de «Mantener proporciones» siguen el ancho real.
- Menú de acciones: cada grupo se abre y cierra por separado
  (`State::toggled_groups`, el primero abierto por defecto) y el scroll llega al
  final aunque haya varios abiertos.
- Bordes: franjas de 8 px y esquinas de 16 px en una capa superior, con cursor de
  redimensionar, y el redimensionado empieza al presionar (antes 5 px, sin
  cursor y tras arrastrar).

### Menú lateral G, fin de Logi y Ajustes en una página

- Interfaz Logi eliminada: `src/logi/`, `theme::logi`, `icons::chrome`, las
  fuentes Noto Sans, `system::set_ui_style`/`stop_daemon`, el selector
  «Interface» de Ajustes, el archivo `interface` (ya no se lee), sus pruebas y
  capturas, `docs/inspiration/openlogi`, `docs/screenshots/logi-dark.png` y 28
  claves de los locales que solo usaba Logi. El «debrand» de textos queda fijo.
- El ítem «Ajustes» del pie del menú lateral ya no muestra el estado USB.
- Ajustes es una sola página con desplazamiento y tres secciones con título y
  tarjeta propia: «Ajustes generales» (servicio, reiniciar, idioma, inicio
  automático, atajos en GNOME Shell), «Administrador de dispositivos» y
  «Copias de seguridad». Cada sección conserva las coordenadas de su antigua
  pestaña, desplazadas a su tarjeta (`settings_layout::SECTION_*`).
- Capturas: `classic-dark-settings(-tall)` reemplaza a las tres por pestaña y
  se revisa a ojo (las referencias HTML tienen pestañas). Se quitó
  `classic-dark-settings-interface-open`.

### Ajustes reordenados y posición en modo mouse

- Selector Dispositivos | Atajos: más espacio entre icono y texto.
- Ajustes: orden Administrador de dispositivos, Ajustes generales, Copias de
  seguridad. La columna «Acciones» de las copias se alinea al borde derecho de
  la tarjeta (antes x fija de 745 que se salía con la barra lateral) y la
  tarjeta ya no deja la fila vacía cuando hay copias. Las capturas
  `settings-general-{es,ja}` incluyen una copia.
- Modo mouse: tabletd no conoce el cursor (lo mueve en relativo) y usaba la
  última posición absoluta del lápiz. Ahora `cursor_point` no envía punto en
  modo mouse ni mueve el puntero al abrir un atajo; la extensión dibuja el
  anillo de «clic derecho, mantener para central» y coloca los menús «junto al
  cursor» con `global.get_pointer()`, y la ventana de atajos fija el menú en la
  primera posición del puntero que recibe.

### Revisión de textos en español

- Corregidas 19 traducciones de `assets/locales/es.json`: etiquetas pendientes
  (Perfil, Orientación, Completa, Copias de seguridad y Acerca de), «Inicio»
  para la página de atajos y «Todas las demás» para el perfil común.
- Unificados «Añadir» y «mouse»; aclaradas las ayudas de presión, doble clic,
  modo del lápiz, cambio de pantalla y extensión de GNOME Shell, además de la
  confirmación de restauración de copias.
- Verificación: `just fix`, `just check` y `just screens` completados.
  Las pruebas de sockets temporales requieren ejecutarse fuera del aislamiento.
  Revisadas visualmente las capturas en español de Lápiz, Ajustes (normal y
  alta), Inicio de atajos y cambio de pantalla; los textos revisados caben.
- `just compare classic-dark-devices-pen classic-dark-devices-display-toggle`:
  RMSE 0,090 y 0,089. Las referencias HTML conservan la distribución anterior
  de navegación y tarjetas; esas diferencias de diseño quedan pendientes y
  fuera de esta corrección de traducciones. No se modificaron medidas ni colores.

### Ayuda del modo mouse sin traducir

- Corregida la clave de traducción de la ayuda del modo mouse: usaba el texto
  inglés revisado en vez de la clave que ya existe en los locales. Se recuperan
  las traducciones existentes para todos los idiomas.
- Añadida una prueba de regresión que comprueba la ayuda en español e inglés y
  una captura en español con modo mouse a 1100×1600 para incluir la tarjeta
  completa, que quedaba fuera de las capturas habituales de 720 px.
- Verificación de la ayuda: `just fix`, `just check` y `just screens` en
  verde; revisada la nueva captura del modo mouse en español. La comparación
  con la referencia HTML conserva las diferencias de diseño ya documentadas.

### README más breve y claro

- README en inglés reducido de 1205 a 750 palabras: presentación directa,
  funciones agrupadas y pasos de instalación más fáciles de seguir.
- Eliminadas repeticiones y afirmaciones desactualizadas sobre las imágenes de
  dispositivos y el umbral visual. Conservadas las instrucciones de instalación,
  desinstalación, acceso a dispositivos y precauciones al modificar el daemon.
- Comprobados los enlaces locales y `git diff --check`. Cambios sin commit,
  por indicación del usuario.

### Página Touch Ring

- Añadida la navegación para Intuos Pro 2013 y la página con cuatro modos,
  conexiones a los LEDs y selector de las seis acciones. El dibujo comparte
  el cálculo de tamaño y posición con ExpressKeys; las posiciones de los LEDs
  están en los tokens del tema. Traducciones nuevas en inglés y español.
- La prueba de interacción comprueba que cambiar el modo 2 conserva los otros
  modos y cierra el menú. Añadidas y revisadas las capturas PTH-451 con y sin
  menú; no existe referencia HTML para estas páginas.
- `just fix` y clippy pasan. `just check` falla en los dos tests de sockets del
  cliente del daemon por `Operation not permitted` del sandbox; el resto del
  workspace pasa al excluir esos dos tests. `just screens` pasa (5 tests).
  Sin commit, por indicación del usuario.
