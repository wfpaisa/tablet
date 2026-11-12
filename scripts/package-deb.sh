#!/usr/bin/env bash
# Builds a .deb package of Tablet Center using standard tar and ar.
set -euo pipefail
cd "$(dirname "$0")/.."

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
arch="amd64"
pkg_dir=$(mktemp -d "${TMPDIR:-/tmp}/tablet-center-deb.XXXXXX")
trap 'rm -rf "$pkg_dir"' EXIT

if [[ ! -f target/release/tabletd || ! -f target/release/tablet-center ]]; then
    cargo build --release -p tabletd -p tablet-center
fi

mkdir -p "$pkg_dir"/{DEBIAN,usr/bin,usr/lib/udev/rules.d,usr/lib/systemd/user,usr/share/applications,usr/share/gnome-shell/extensions/tablet-center@tabletcenter.dev} dist

cat > "$pkg_dir/DEBIAN/control" <<EOF
Package: tablet-center
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Maintainer: Tablet Center Developers <https://github.com/wfpaisa/tablet>
Description: Wacom tablet settings and daemon for GNOME Wayland
 Tablet Center lets you configure your Wacom tablet on GNOME Wayland.
 Adjust pen pressure, buttons, gestures, and shortcut menus.
EOF

cat > "$pkg_dir/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
udevadm control --reload 2>/dev/null || true
udevadm trigger 2>/dev/null || true
gtk-update-icon-cache -q -t /usr/share/icons/hicolor 2>/dev/null || true
update-desktop-database -q /usr/share/applications 2>/dev/null || true
EOF
chmod 755 "$pkg_dir/DEBIAN/postinst"

install -m 755 target/release/tabletd "$pkg_dir/usr/bin/tabletd"
install -m 755 target/release/tablet-center "$pkg_dir/usr/bin/tablet-center"
install -m 644 packaging/70-tablet-center.rules "$pkg_dir/usr/lib/udev/rules.d/70-tablet-center.rules"
install -m 644 packaging/tabletd.service "$pkg_dir/usr/lib/systemd/user/tabletd.service"
install -m 644 packaging/tablet-center.desktop "$pkg_dir/usr/share/applications/tablet-center.desktop"

for size in 48 64 128 256 512; do
    dir="$pkg_dir/usr/share/icons/hicolor/${size}x$size/apps"
    mkdir -p "$dir"
    install -m 644 "packaging/icons/tablet-center-$size.png" "$dir/tablet-center.png"
done

install -m 644 gnome-extension/tablet-center@tabletcenter.dev/* "$pkg_dir/usr/share/gnome-shell/extensions/tablet-center@tabletcenter.dev/"

deb_file="dist/tablet-center_${version}_${arch}.deb"
tar -czf "$pkg_dir/control.tar.gz" -C "$pkg_dir/DEBIAN" control postinst
tar -czf "$pkg_dir/data.tar.gz" -C "$pkg_dir" usr
echo "2.0" > "$pkg_dir/debian-binary"

ar rc "$deb_file" "$pkg_dir/debian-binary" "$pkg_dir/control.tar.gz" "$pkg_dir/data.tar.gz"
echo "Created DEB package: $deb_file"
