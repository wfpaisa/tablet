Name:           tablet-center
Version:        1.3.2
Release:        1%{?dist}
Summary:        Wacom tablet settings and daemon for GNOME Wayland
License:        GPL-3.0-or-later
URL:            https://github.com/wfpaisa/tablet

%description
Tablet Center lets you configure your Wacom tablet on GNOME Wayland. Adjust pen
pressure, buttons, gestures, and shortcut menus.

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 %{_sourcedir}/tabletd %{buildroot}%{_bindir}/tabletd
install -m 755 %{_sourcedir}/tablet-center %{buildroot}%{_bindir}/tablet-center

mkdir -p %{buildroot}%{_udevrulesdir}
install -m 644 %{_sourcedir}/packaging/70-tablet-center.rules %{buildroot}%{_udevrulesdir}/70-tablet-center.rules

mkdir -p %{buildroot}%{_userunitdir}
install -m 644 %{_sourcedir}/packaging/tabletd.service %{buildroot}%{_userunitdir}/tabletd.service

mkdir -p %{buildroot}%{_datadir}/applications
install -m 644 %{_sourcedir}/packaging/tablet-center.desktop %{buildroot}%{_datadir}/applications/tablet-center.desktop

for size in 48 64 128 256 512; do
    mkdir -p %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps
    install -m 644 %{_sourcedir}/packaging/icons/tablet-center-${size}.png %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/tablet-center.png
done

mkdir -p %{buildroot}%{_datadir}/gnome-shell/extensions/tablet-center@tabletcenter.dev
install -m 644 %{_sourcedir}/gnome-extension/tablet-center@tabletcenter.dev/* %{buildroot}%{_datadir}/gnome-shell/extensions/tablet-center@tabletcenter.dev/

%files
%{_bindir}/tabletd
%{_bindir}/tablet-center
%{_udevrulesdir}/70-tablet-center.rules
%{_userunitdir}/tabletd.service
%{_datadir}/applications/tablet-center.desktop
%{_datadir}/icons/hicolor/*/apps/tablet-center.png
%{_datadir}/gnome-shell/extensions/tablet-center@tabletcenter.dev
