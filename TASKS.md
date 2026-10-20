# Tasks

Features of the Wacom driver for macOS (Wacom Center 6.4.15) that Tablet
Center does not have yet, limited to what applies to a pen tablet like the
Intuos S (no touch, eraser, dials, Touch Ring, pen display calibration or
Cintiq Workflows).

## High priority

- [ ] **Per-application settings**: different pen buttons and ExpressKeys
  for each program (Krita, GIMP, Blender…). The GNOME Shell extension
  reports the focused window to `tabletd`, which switches to that app's
  profile. The largest task.
- [x] **Tablet orientation**: landscape, portrait, landscape flipped and
  portrait flipped, with rotated pen coordinates, gestures and tablet diagrams.
- [ ] **Precision mode**: while the button is held, the tablet covers only a
  small area around the cursor for fine work. A coordinate change in
  `mapping.rs` and a new button action.
- [ ] **Portion of tablet / portion of screen**: use part of the tablet, or
  map it to part of a screen (only "Use full area" exists today). A
  draggable rectangle on the Mapping page.

## Medium priority: new actions for pen buttons and ExpressKeys

- [ ] **Keyboard modifier**: hold Ctrl, Shift or Alt while the button is held.
- [ ] **Open application**: launch a program or file.
- [ ] **Mode toggle**: switch between pen mode and mouse mode.
- [ ] **Relative mouse mode**: the pen moves the cursor like a mouse, with a
  speed setting, instead of jumping to the position. The current "Use the
  pen as a mouse" is absolute.
- [ ] **Pressure hold**: freeze the current pressure while the button is held.
- [ ] **Click lock**: keep the click pressed until the next click, to drag
  without holding the tip down.
- [ ] **System actions**: show desktop, Activities overview, switch
  application, notification center, on-screen keyboard.

## Low priority

- [ ] **Double-click distance**: how far the pen may move between the two
  taps of a double click.
- [ ] **Tip double-click / hover click**: fine control over when a tip or
  button click counts.
- [ ] **Battery**: charge level for the Intuos S Bluetooth (CTL-4100WL).
- [ ] **Device info**: firmware, serial number, live tilt and pressure, for
  diagnostics.
- [ ] **More tablets**: artwork and key layouts for Intuos M, Intuos Pro and
  One by Wacom (only the Intuos S is drawn today).
