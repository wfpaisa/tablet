# App icons

Every icon Tablet Center draws, one SVG per role. To change an icon, replace
the file with another 24×24 SVG that uses `fill="currentColor"` (or
`stroke="currentColor"`): the app swaps `currentColor` for white and tints it.
A new role needs a line in `roles!` in `crates/tablet-center/src/icons.rs`.

Source: `/home/projects/ia/icons/svg` (Hugeicons Stroke Rounded glyphs converted
to SVG paths by `/home/projects/ia/icons/scripts/font_to_svg.py`).

| File | Source icon | Used for |
| --- | --- | --- |
| `left-click.svg` | `mouse-left-click-03` | Actions: click, click lock, pressure hold |
| `right-click.svg` | `mouse-right-click-03` | Actions: right click, right click lock |
| `middle-click.svg` | `mouse-03` | Actions: middle click, right/hold middle, pen/mouse toggle |
| `double-click.svg` | `mouse-pointer-click` | Action: double click |
| `pan.svg` | `move` | Actions: pan/scroll (and legacy) |
| `zoom.svg` | `zoom-in` | Actions: pan/zoom, zoom |
| `scroll.svg` | `arrow-up-down` | Action: scroll |
| `wheel.svg` | `refresh-01` | Action: wheel |
| `back.svg` | `arrow-left-02` | Action: back |
| `forward.svg` | `arrow-right-02` | Action: forward |
| `undo.svg` | `undo-03` | Action and menu item: undo |
| `redo.svg` | `redo-03` | Action and menu item: redo |
| `copy.svg` | `copy-01` | Action and menu item: copy |
| `paste.svg` | `clipboard-paste` | Action and menu item: paste |
| `cut.svg` | `scissors` | Action: cut |
| `save.svg` | `floppy-disk` | Action and menu item: save |
| `open.svg` | `folder-01` | Action: open |
| `select-all.svg` | `check-list` | Action: select all |
| `type.svg` | `type` | Action: type text |
| `modifier.svg` | `command` | Action: modifier |
| `run.svg` | `command-line` | Action: run command |
| `disabled.svg` | `unavailable` | Action: disabled |
| `default.svg` | `undo` | Action: default |
| `mode-toggle.svg` | `refresh-cw` | Action: mode toggle |
| `precision.svg` | `crosshair` | Action: precision mode |
| `display.svg` | `monitor` | Actions: display toggle, show desktop |
| `overview.svg` | `layers-01` | Action: overview |
| `switch-app.svg` | `app-window` | Action: switch application |
| `notifications.svg` | `notification-01` | Action: notification center |
| `on-screen-menu.svg` | `circle-dot` | Actions and sidebar: express menu, radial menu |
| `app-shortcuts.svg` | `grid-view` | Action and sidebar: app shortcuts |
| `keypad.svg` | `dialpad-square-01` | Action and sidebar: keypad |
| `keystroke.svg` | `keyboard` | Keyboard shortcuts; fallback for unknown roles |
| `nav-devices.svg` | `tablet-pen` | Top switch: Devices |
| `nav-shortcuts.svg` | `grid-view` | Top switch: Shortcuts |
| `nav-settings.svg` | `settings-01` | Sidebar footer: Settings |
| `nav-pen.svg` | `pen-01` | Sidebar and overview: Pen; Device info "Pen input" card |
| `nav-express-keys.svg` | `keyboard` | Sidebar and overview: ExpressKeys |
| `nav-mapping.svg` | `monitor` | Sidebar and overview: Mapping |
| `nav-display-toggle.svg` | `refresh-cw` | Sidebar and overview: Display toggle / Touch Ring |
| `nav-touch.svg` | `hand` | Sidebar: Touch |
| `home.svg` | `home-01` | Shortcuts sidebar: Start |
| `gesture.svg` | `move` | Shortcuts sidebar: pen gestures |
| `close.svg` | `cancel-01` | Radial menu center, action panel close, remove app |
| `info.svg` | `information-circle` | Device info button and card |
| `usb.svg` | `usb` | Overview connection badge |
| `add.svg` | `plus-sign` | Applications strip: Add |
| `previous.svg` | `chevron-left` | Applications strip: scroll left |
| `next.svg` | `chevron-right` | Applications strip, Device info button, callouts |
| `select-check.svg` | `tick-02` | Selected row in dropdowns (language, touch ring) |
| `window-minimize.svg` | `minus-sign` | Title bar: minimize |
| `window-maximize.svg` | `square` | Title bar: maximize |
| `window-close.svg` | `cancel-01` | Title bar: close |
