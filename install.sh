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
# Output: one line per step with a spinner while it runs, then ✓ or ✗.
# Command output goes to a log that is shown only when a step fails.
log=$(mktemp "${TMPDIR:-/tmp}/tablet-center-install.XXXXXX")
if [ -t 1 ]; then tty=true; else tty=false; fi
if $tty && [ -z "${NO_COLOR:-}" ]; then
	bold=$(printf '\033[1m') dim=$(printf '\033[2m') reset=$(printf '\033[0m')
	green=$(printf '\033[32m') red=$(printf '\033[31m') yellow=$(printf '\033[33m') blue=$(printf '\033[34m')
else
	bold='' dim='' reset='' green='' red='' yellow='' blue=''
fi
$tty && printf '\033[?25l'
trap '$tty && printf "\033[?25h"' EXIT
trap 'exit 130' INT TERM

done_line() {
	$tty && printf '\r\033[K'
	printf '  %s %s\n' "$1" "$2"
}
fail() {
	done_line "${red}✗${reset}" "$1"
	echo
	echo "  ${red}${bold}Installation failed.${reset} Last lines of the log:"
	tail -n 15 "$log" | sed 's/^/    /'
	echo "  ${dim}Full log: $log${reset}"
	exit 1
}

# step LABEL COMMAND...: runs COMMAND in the background behind a spinner.
step() {
	label=$1; shift
	"$@" >>"$log" 2>&1 &
	pid=$!
	if $tty; then
		i=0
		while kill -0 "$pid" 2>/dev/null; do
			case $i in
			0) f=⠋ ;; 1) f=⠙ ;; 2) f=⠹ ;; 3) f=⠸ ;; 4) f=⠼ ;;
			5) f=⠴ ;; 6) f=⠦ ;; 7) f=⠧ ;; 8) f=⠇ ;; *) f=⠏ ;;
			esac
			printf '\r\033[K  %s %s' "${blue}$f${reset}" "$label"
			i=$(((i + 1) % 10))
			sleep 0.08
		done
	fi
	if wait "$pid"; then done_line "${green}✓${reset}" "$label"; else fail "$label"; fi
}
note() { printf '  %s %s\n' "${dim}·${reset}" "${dim}$1${reset}"; }

echo
echo "  ${bold}Tablet Center${reset} ${dim}installer${reset}"
echo

case "${XDG_CURRENT_DESKTOP:-GNOME}" in
*GNOME*) ;;
*) echo "  ${red}✗${reset} Tablet Center needs GNOME on Wayland; this session is $XDG_CURRENT_DESKTOP." >&2; exit 1 ;;
esac

if [ -x bin/tabletd ] && [ -x bin/tablet-center ]; then
	bin=bin
	note "Using the prebuilt binaries in bin/"
else
	command -v cargo >/dev/null || { echo "  ${red}✗${reset} cargo not found: install Rust, or use a release archive." >&2; exit 1; }
	step "Building Tablet Center ${dim}(the first build takes a few minutes)${reset}" \
		cargo build -q --release -p tabletd -p tablet-center
	bin=target/release
fi

# System part, only when something is missing: uinput access and the input
# group.
rule=/etc/udev/rules.d/70-tablet-center.rules
need_rule=false
cmp -s packaging/70-tablet-center.rules "$rule" 2>/dev/null || need_rule=true
need_group=false
id -nG | tr ' ' '\n' | grep -qx input || need_group=true
if $need_rule || $need_group; then
	note "Administrator rights are needed once to set up the system."
	$tty && printf '\033[?25h'
	sudo -v || fail "Getting administrator rights"
	$tty && printf '\033[?25l'
	install_rule() {
		sudo -n install -m 644 packaging/70-tablet-center.rules "$rule"
		sudo -n udevadm control --reload
		sudo -n udevadm trigger --subsystem-match=misc --sysname-match=uinput
	}
	$need_rule && step "Allowing access to virtual input devices" install_rule
	$need_group && step "Adding you to the input group" sudo -n usermod -aG input "$USER"
fi

# Stopping tabletd removes its virtual pen, which can crash GNOME Shell while
# the real pen hovers. tabletd waits for the pen to leave on its own; an older
# build without that option falls back to asking.
if grep -qa 'Move the pen away from the tablet' "$bin/tabletd" 2>/dev/null; then
	"$bin/tabletd" --wait-pen-away >>"$log" 2>&1 &
	pid=$!
	if $tty; then
		while kill -0 "$pid" 2>/dev/null; do
			if grep -q 'Move the pen away' "$log"; then
				printf '\r\033[K  %s %s' "${yellow}●${reset}" "${yellow}${bold}Move the pen away from the tablet…${reset}"
			else
				printf '\r\033[K  %s %s' "${blue}⠿${reset}" "Checking the pen is away from the tablet"
			fi
			sleep 0.1
		done
	fi
	wait "$pid" || true
	done_line "${green}✓${reset}" "Pen is away from the tablet"
elif [ -t 0 ]; then
	$tty && printf '\033[?25h'
	printf '  %s Move the pen away from the tablet, then press Enter. ' "${yellow}●${reset}"
	read -r _
	$tty && printf '\033[?25l'
fi

install_service() {
	mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
	install -m 755 "$bin/tabletd" "$HOME/.local/bin/tabletd"
	install -m 755 "$bin/tablet-center" "$HOME/.local/bin/tablet-center"
	install -m 644 packaging/tabletd.service "$HOME/.config/systemd/user/tabletd.service"
	systemctl --user daemon-reload
	systemctl --user enable tabletd.service
	systemctl --user restart tabletd.service
}
step "Installing and starting the tabletd service" install_service

install_app() {
	apps="$HOME/.local/share/applications"
	mkdir -p "$apps"
	sed "s|^Exec=.*|Exec=$HOME/.local/bin/tablet-center|" packaging/tablet-center.desktop >"$apps/tablet-center.desktop"
	for size in 48 64 128 256 512; do
		dir="$HOME/.local/share/icons/hicolor/${size}x$size/apps"
		mkdir -p "$dir"
		install -m 644 "packaging/icons/tablet-center-$size.png" "$dir/tablet-center.png"
	done
	# Early builds also installed a scalable icon; GNOME prefers it over the PNGs.
	rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/tablet-center.svg"
	gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
	update-desktop-database -q "$apps" 2>/dev/null || true
}
step "Adding Tablet Center to your applications" install_app

# GNOME Shell extension: the hold ring and on-screen shortcuts.
ext=tablet-center@tabletcenter.dev
ext_dir="$HOME/.local/share/gnome-shell/extensions/$ext"
new_ext=true
[ -d "$ext_dir" ] && new_ext=false
install_extension() {
	mkdir -p "$ext_dir"
	install -m 644 gnome-extension/$ext/* "$ext_dir/"
	enabled=$(gsettings get org.gnome.shell enabled-extensions)
	case "$enabled" in
	*"'$ext'"*) ;;
	"@as []") gsettings set org.gnome.shell enabled-extensions "['$ext']" ;;
	*) gsettings set org.gnome.shell enabled-extensions "${enabled%]}, '$ext']" ;;
	esac
}
step "Installing the GNOME Shell extension" install_extension

echo
if systemctl --user is-active -q tabletd.service; then
	echo "  ${green}${bold}Tablet Center is installed.${reset} ${dim}tabletd is running.${reset}"
else
	echo "  ${yellow}${bold}Tablet Center is installed, but tabletd is not running.${reset}"
	echo "  ${dim}See: journalctl --user -u tabletd${reset}"
fi
echo
if $need_group; then
	echo "  ${bold}Next:${reset} log out and back in. You were added to the input group,"
	echo "  and GNOME Shell loads the extension then. If the tablet still does"
	echo "  nothing after that, restart the computer."
elif $new_ext; then
	echo "  ${bold}Next:${reset} log out and back in once so GNOME Shell loads the extension."
else
	echo "  ${dim}GNOME Shell uses the updated extension after logging out and back in.${reset}"
fi
echo
rm -f "$log"
