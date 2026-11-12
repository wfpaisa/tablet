#!/usr/bin/env bash
# Builds an .rpm package of Tablet Center using rpmbuild.
set -euo pipefail
cd "$(dirname "$0")/.."

command -v rpmbuild >/dev/null || { echo "package-rpm: rpmbuild not found" >&2; exit 1; }

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
topdir=$(mktemp -d "${TMPDIR:-/tmp}/tablet-center-rpmbuild.XXXXXX")
trap 'rm -rf "$topdir"' EXIT

mkdir -p "$topdir"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS} dist

if [[ ! -f target/release/tabletd || ! -f target/release/tablet-center ]]; then
    cargo build --release -p tabletd -p tablet-center
fi

cp target/release/tabletd target/release/tablet-center "$topdir/SOURCES/"
cp -r packaging gnome-extension "$topdir/SOURCES/"
sed "s/^Version:.*/Version:        $version/" packaging/tablet-center.spec > "$topdir/SPECS/tablet-center.spec"

rpmbuild -bb --quiet --define "_topdir $topdir" "$topdir/SPECS/tablet-center.spec"

cp "$topdir"/RPMS/*/*.rpm dist/
echo "RPM package created in dist/:"
ls -lh dist/*.rpm
