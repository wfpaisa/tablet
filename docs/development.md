# Desarrollo

Las tareas del proyecto están en el `justfile` (equivalente a los «scripts» de
`package.json`). Hace falta [`just`](https://github.com/casey/just)
(`pacman -S just`, `apt install just` o `cargo install just`).

```sh
just            # lista todas las tareas
just <tarea>    # ejecuta una
```

## Ejecutar

| Tarea | Qué hace |
|---|---|
| `just dev` | Abre Tablet Center (debug) contra el `tabletd` instalado. |
| `just dev-sim` | Abre la app con datos simulados (Intuos S, dos monitores Dell): sin daemon y sin tocar `~/.config/tablet-center`. `just dev-sim 0314` simula otro modelo por su id USB (hex): 0314/0315/0317 = Intuos Pro 2013 S/M/L. |
| `just daemon` | Corre un `tabletd` local. Antes hay que detener `tabletd.service` si está activo. |
| `just run` | Abre la app en release. |

## Calidad

| Tarea | Qué hace |
|---|---|
| `just test` | Tests de todo el workspace. |
| `just lint` | Comprueba el formato de `tablet-center` y ejecuta clippy en todo el workspace con `-D warnings`. |
| `just fix` | Formatea `tablet-center`; conserva el formato de los crates copiados. |
| `just check` | `lint` + `test`: todo lo que debe estar en verde antes de commitear. |
| `just smoke-shortcuts` | Prueba las ventanas en pantalla en un servidor X aislado. |

## Revisión visual

La app egui debe verse igual que la UI HTML original
(`/home/projects/ia/wacom/app/ui`). Las capturas de referencia están en
`docs/reference/` (`classic-*` para la interfaz Tablet; a 1100×720 y `-tall` a
1100×1600). Ajustes es ahora una sola página con secciones y ya no se compara
con las referencias por pestañas: `classic-dark-settings(-tall)` se revisa a ojo.

| Tarea | Qué hace |
|---|---|
| `just screens` | Genera las capturas de la app en `target/screens` con `egui_kittest` (necesita GPU). Incluye `classic-dark-es-*`, sin referencia, para revisar textos largos en español. |
| `just compare [nombres…]` | Pone lado a lado referencia, captura y mapa de diferencias en `target/compare/` e imprime el RMSE. Sin nombres compara todas. |
| `just review` | `screens` + `compare` de todo. |
| `just reference` | Regenera `docs/reference/` desde la UI original con Chromium headless (levanta su dev-server en :8765 si hace falta). |
| `just reference-settings` | Capturas extra de Settings del original en `target/settings-reference` (selects, toasts, diálogos, es/ja; requiere Node con WebSocket, Chromium y Python 3). |

Criterio: RMSE ≤ 0,03 por captura y sin diferencias visibles de composición,
tamaños, colores, iconos ni textos. El resto es suavizado de fuentes y SVG.

## Empaquetar e instalar

| Tarea | Qué hace |
|---|---|
| `just build` | Build de release de `tablet-center` y `tabletd`. |
| `just install` | Instala para el usuario actual (servicio, reglas udev, extensión de GNOME). |
| `just uninstall [--purge]` | Desinstala; `--purge` borra también la configuración. |
| `just package` | Genera `dist/tablet-center-<arch>-linux.tar.gz`. |
| `just fonts` | Rehace las fuentes Roboto embebidas desde los WOFF2 del original. |
| `just clean` | Borra los artefactos de compilación. |

Para comparar las capturas adicionales con el HTML original:

```sh
just reference-settings
REFERENCE_DIR=target/settings-reference just compare classic-dark-settings-language-open classic-dark-settings-toast classic-dark-settings-reset-confirmation classic-dark-settings-delete-confirmation classic-dark-settings-create-dialog classic-dark-settings-general-es classic-dark-settings-general-es-tall classic-dark-settings-general-ja classic-dark-settings-general-ja-tall
```

Las referencias adicionales se generan en `target`; no reemplazan las capturas
versionadas de `docs/reference`. El perfil temporal de Chromium también queda
en `target`, sin usar el perfil personal.

La ventana conserva sus esquinas transparentes en ejecución nativa. Roboto
conserva su configuración de las fases anteriores.
