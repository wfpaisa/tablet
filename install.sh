#!/bin/sh
# Installs Tablet Center for the current user: the tabletd service that
# starts with the graphical session, the settings app and its GNOME Shell
# extension. From a release archive it uses the binaries in bin/; from a
# source checkout it builds them with cargo first.
#
# Keep the pen away from the tablet while this runs: restarting tabletd
# replaces its virtual devices, and GNOME Shell can crash if that happens
# while the pen is in proximity.
set -eu
cd "$(dirname "$0")"

case "${XDG_CURRENT_DESKTOP:-GNOME}" in
*GNOME*) ;;
*) echo "Tablet Center needs GNOME on Wayland; this session is $XDG_CURRENT_DESKTOP." >&2; exit 1 ;;
esac

if [ -x bin/tabletd ] && [ -x bin/tablet-center ]; then
	bin=bin
else
	command -v cargo >/dev/null || { echo "cargo not found: install Rust, or use a release archive." >&2; exit 1; }
	cargo build -q --release -p tabletd -p tablet-center
	bin=target/release
fi

# System part, only when something is missing: the WebKitGTK library the app
# window needs, uinput access and the input group.
webkit=""
if ! ldconfig -p 2>/dev/null | grep -q 'libwebkit2gtk-4\.1\.so\.0'; then
	if command -v apt-get >/dev/null; then webkit="apt-get install -y libwebkit2gtk-4.1-0"
	elif command -v dnf >/dev/null; then webkit="dnf install -y webkit2gtk4.1"
	elif command -v pacman >/dev/null; then webkit="pacman -S --needed --noconfirm webkit2gtk-4.1"
	elif command -v zypper >/dev/null; then webkit="zypper --non-interactive install libwebkit2gtk-4_1-0"
	else echo "Install WebKitGTK 4.1 with your package manager, then run this again." >&2; exit 1
	fi
fi
rule=/etc/udev/rules.d/70-tablet-center.rules
need_rule=false
cmp -s packaging/70-tablet-center.rules "$rule" 2>/dev/null || need_rule=true
need_group=false
id -nG | tr ' ' '\n' | grep -qx input || need_group=true
if [ -n "$webkit" ] || $need_rule || $need_group; then
	echo "Administrator rights are needed once to set up the system (sudo)."
	[ -n "$webkit" ] && sudo $webkit
	if $need_rule; then
		sudo install -m 644 packaging/70-tablet-center.rules "$rule"
		sudo udevadm control --reload
		sudo udevadm trigger --subsystem-match=misc --sysname-match=uinput
	fi
	$need_group && sudo usermod -aG input "$USER"
fi

if [ -t 0 ]; then
	printf 'Move the pen away from the tablet, then press Enter. '
	read -r _
fi

mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
install -m 755 "$bin/tabletd" "$HOME/.local/bin/tabletd"
install -m 755 "$bin/tablet-center" "$HOME/.local/bin/tablet-center"
install -m 644 packaging/tabletd.service "$HOME/.config/systemd/user/tabletd.service"
systemctl --user daemon-reload
systemctl --user enable tabletd.service
systemctl --user restart tabletd.service

apps="$HOME/.local/share/applications"
mkdir -p "$apps"
sed "s|^Exec=.*|Exec=$HOME/.local/bin/tablet-center|" packaging/tablet-center.desktop >"$apps/tablet-center.desktop"
for size in 48 64 128 256 512; do
	dir="$HOME/.local/share/icons/hicolor/${size}x$size/apps"
	mkdir -p "$dir"
	install -m 644 "packaging/icons/tablet-center-$size.png" "$dir/tablet-center.png"
done
gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
update-desktop-database -q "$apps" 2>/dev/null || true

# GNOME Shell extension: the hold ring and on-screen shortcuts.
ext=tablet-center@tabletcenter.dev
ext_dir="$HOME/.local/share/gnome-shell/extensions/$ext"
new_ext=true
[ -d "$ext_dir" ] && new_ext=false
mkdir -p "$ext_dir"
install -m 644 gnome-extension/$ext/* "$ext_dir/"
enabled=$(gsettings get org.gnome.shell enabled-extensions)
case "$enabled" in
*"'$ext'"*) ;;
"@as []") gsettings set org.gnome.shell enabled-extensions "['$ext']" ;;
*) gsettings set org.gnome.shell enabled-extensions "${enabled%]}, '$ext']" ;;
esac

echo
systemctl --user --no-pager status tabletd.service | head -3
if $need_group; then
	echo "Log out and back in: you were added to the input group, and GNOME Shell loads the extension then."
	echo "If the tablet still does nothing after that, restart the computer."
elif $new_ext; then
	echo "Log out and back in once so GNOME Shell loads the extension."
else
	echo "Installed. GNOME Shell uses the updated extension after logging out and back in."
fi
