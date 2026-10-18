#!/bin/sh
# Removes Tablet Center from the current user. Settings and backups in
# ~/.config/tablet-center stay unless you pass --purge. Keep the pen away
# from the tablet: stopping tabletd removes its virtual devices.
set -eu

if [ -t 0 ]; then
	printf 'Move the pen away from the tablet, then press Enter. '
	read -r _
fi

systemctl --user disable --now tabletd.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/tabletd.service"
systemctl --user daemon-reload
rm -f "$HOME/.local/bin/tabletd" "$HOME/.local/bin/tablet-center" \
	"$HOME/.local/share/applications/tablet-center.desktop" \
	"$HOME/.config/autostart/tablet-center.desktop"
for size in 48 64 128 256 512; do
	rm -f "$HOME/.local/share/icons/hicolor/${size}x$size/apps/tablet-center.png"
done
gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

ext=tablet-center@tabletcenter.dev
rm -rf "$HOME/.local/share/gnome-shell/extensions/$ext"
enabled=$(gsettings get org.gnome.shell enabled-extensions)
case "$enabled" in
*"'$ext'"*)
	enabled=$(printf '%s' "$enabled" | sed "s/, '$ext'//; s/'$ext', //; s/'$ext'//")
	[ "$enabled" = "[]" ] && enabled="@as []"
	gsettings set org.gnome.shell enabled-extensions "$enabled"
	;;
esac

if [ "${1:-}" = "--purge" ]; then
	rm -rf "$HOME/.config/tablet-center" "$HOME/.local/share/dev.tabletcenter.linux"
fi

if [ -f /etc/udev/rules.d/70-tablet-center.rules ]; then
	echo "Removing the udev rule needs administrator rights (sudo)."
	sudo rm -f /etc/udev/rules.d/70-tablet-center.rules
	sudo udevadm control --reload
fi
echo "Tablet Center removed. You stay in the input group; leave it with: sudo gpasswd -d $USER input"
