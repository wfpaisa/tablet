# Tablet Center: Wacom settings for Linux

Configure your Wacom tablet on GNOME Wayland. Adjust the pen, buttons and
screens, scroll with pen gestures, and open shortcut menus while you work.
Changes apply immediately.

Tested with the Intuos S (CTL-4100). Other Wacom tablets may work.
Built in Rust with egui.

> Unofficial project, not affiliated with or endorsed by Wacom. Wacom,
> Intuos and ExpressKeys are trademarks of Wacom Co., Ltd.

![Pen settings](docs/screenshots/pen-settings.png)

## Install

Requires GNOME 48 or later on Wayland and an x86_64 PC.
Keep the pen away from the tablet, then choose one of the installation methods:

### Option 1: Quick install from GitHub release

```sh
curl -fsSL https://github.com/wfpaisa/tablet/releases/latest/download/tablet-center-x86_64-linux.tar.gz | tar xz && ./tablet-center/install.sh
```

### Option 2: Install RPM package (Fedora / RHEL / openSUSE)

Download `tablet-center-*.rpm` from the [latest release](https://github.com/wfpaisa/tablet/releases/latest), then run:

```sh
sudo dnf install ./tablet-center-*.rpm
```

### Option 3: Install DEB package (Debian / Ubuntu)

Download `tablet-center_*.deb` from the [latest release](https://github.com/wfpaisa/tablet/releases/latest), then run:

```sh
sudo apt install ./tablet-center_*.deb
```

### Option 4: Build from source / Remake anytime

Install stable [Rust](https://rustup.rs), clone this repository, then run:

```sh
./install.sh
# or if you have just installed:
just build && just install
```

When installation finishes, enter your password if prompted for the udev rule,
then log out and back in so the `input` group and GNOME Shell extension take
effect. If the tablet still does not respond, restart the computer.

To uninstall, run `./uninstall.sh` (or `just uninstall`). Add `--purge` to
remove your settings too.

## Features

- **Pen settings:** adjust pressure, click sensitivity and double-click
  distance. Choose pen or mouse mode.
- **Button actions:** assign keyboard shortcuts, clicks, gestures or menus
  to pen buttons and ExpressKeys.
- **Scrolling and zoom:** hold a pen button and move the pen to scroll,
  pan or zoom. Adjust the speed and direction.
- **Shortcut menus:** open a radial menu, a grid of app shortcuts or a
  numeric keypad from a tablet button.
- **Screen mapping:** use one screen or all screens, limit the active area,
  and keep the pen movement proportional to the screen.
- **App profiles:** use different settings for each application.
- **Right click, hold for middle:** tap for a right click; hold or move the
  pen for a middle click.
- **Settings:** choose from 14 languages, start the app at login, and back
  up or restore your configuration. English is the default language.

The included GNOME Shell extension can show shortcut menus over your current
application without switching focus. Enable this in **Settings**. Without
it, menus open in a separate window.

## Screenshots

| | |
|---|---|
| ![ExpressKeys](docs/screenshots/expresskeys-settings.png) | ![Express menu](docs/screenshots/express-menu-settings.png) |
| Tablet button actions | On-screen shortcut menu |
| ![Pan/Scroll](docs/screenshots/pan-scroll-settings.png) | ![Mapping](docs/screenshots/mapping-settings.png) |
| Scroll with the pen | Choose the screens your tablet controls |

## Installation details

The installer adds the app, a background service (`tabletd`) and a GNOME
Shell extension to your home folder. The service starts with your session.
Your password is needed to set up tablet access: a udev rule for
`/dev/uinput` and membership in the `input` group.

To check the service:

```sh
systemctl --user status tabletd
```

GNOME is required because the service reads screen information from Mutter.
Other desktops are not supported.

### Build from source

Install stable [Rust](https://rustup.rs), clone this repository, then run:

```sh
./install.sh
```

This builds and installs the app, service and extension. No WebKitGTK is
required. Keep the pen away from the tablet during installation.

## How it works

`tabletd` reads the tablet input, applies your settings, and sends pen,
mouse and keyboard events to GNOME through virtual input devices.
Tablet Center is the settings app.

| Component | Purpose |
|---|---|
| `crates/tablet-center` | Settings window, built with egui and wgpu |
| `crates/tabletd` | Background service that handles tablet input |
| `crates/tablet-config` | Shared settings and communication format |

Settings are stored in `~/.config/tablet-center/config.json`. The app and
service communicate through `$XDG_RUNTIME_DIR/tabletd.sock`.

## Development

Install [`just`](https://github.com/casey/just) to run the project tasks.
Run `just` to list them, or see [docs/development.md](docs/development.md)
for the full workflow.

```sh
just build        # build release binaries for tabletd and tablet-center
just install      # install binaries, systemd service, udev rules and extension
just dev          # open the app with the installed service
just dev-sim      # try simulated data without changing your settings
just dev-sim 037a # simulate a specific model (e.g. 037a = One by Wacom CTL-472)
just check        # check formatting, code quality and tests
just package      # create portable release archive in dist/ (.tar.gz)
just package-rpm  # build RPM package in dist/ (Fedora/RHEL)
just package-deb  # build DEB package in dist/ (Debian/Ubuntu)
just release      # bump version, build all packages, commit, tag and publish GitHub release
```

### Packaging and GitHub Releases

To generate release packages locally into `dist/`:

```sh
# Portable tarball (.tar.gz)
just package
# or without containers:
./scripts/package.sh --local

# RPM package (.rpm)
just package-rpm

# DEB package (.deb)
just package-deb
```

To create a new GitHub release with all packages attached:

```sh
just release          # bumps patch version (e.g. 1.3.2 -> 1.3.3) and publishes
just release 1.4.0    # releases a specific version
```

This runs quality checks, packages the `.tar.gz`, `.rpm`, and `.deb` archives,
creates a git tag, pushes to GitHub, and publishes the release assets via `gh`.

### Simulating a model

`just dev-sim <id>` opens the app as if that tablet were connected, using
its USB product ID in hex (for example `just dev-sim 0314`). Without an ID
it simulates an Intuos S (`0374`). Each listed model uses its own tablet
artwork and the corresponding Intuos Pen, Wacom One Pen, Pro Pen 2 or Grip Pen.
Intuos Pro 2013 uses its ExpressKeys panel on the key configuration page.

| Model | Product IDs |
|---|---|
| Intuos S CTL-4100 | `0374` |
| Intuos M CTL-6100 | `0375` |
| Intuos S CTL-4100WL | `0376`, `0377`, `03c6` |
| Intuos M CTL-6100WL | `0378`, `0379`, `03c8` |
| One by Wacom S CTL-472 | `037a` |
| Wacom One S CTC4110WL | `0100`, `0101`, `0104` |
| Wacom One M CTC6110WL | `0102`, `0103`, `0105` |
| Wacom One 12 DTC121 | `03ce` |
| Wacom One 13 touch DTH134 | `03cb` |
| Intuos Pro S PTH-460 | `0392`, `0393`, `03dd` |
| Intuos Pro M PTH-660 | `0357`, `0360` |
| Intuos Pro L PTH-860 | `0358`, `0361` |
| Intuos Pro S PTH-451 (2013) | `0314` |
| Intuos Pro M PTH-651 (2013) | `0315` |
| Intuos Pro L PTH-851 (2013) | `0317` |

Review screenshots at 1100×720 and 1100×1600 against `docs/reference/`.
Check text, layout, colors and icons visually as well as the reported image
differences.

**When changing `tabletd`:** Mutter can crash and end the session if a
virtual tablet is removed or changes owner while the pen is near the tablet.
Preserve the daemon's safeguards: release inputs before removing devices,
wait until the pen is away before taking control, and never reuse the real
pen's serial number. Stop the daemon with SIGTERM, not SIGKILL.

## Credits

- Fonts: [Roboto](https://github.com/googlefonts/roboto-classic), SIL Open
  Font License.
- App icon: created for this project.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
