#!/bin/sh
# Builds the release binaries and packs them with the installer into
# dist/tablet-center-<arch>-linux.tar.gz, the archive attached to GitHub
# releases. The name has no version so the README can link to
# releases/latest/download/. Usage: scripts/package.sh
set -eu
cd "$(dirname "$0")/.."

name="tablet-center-$(uname -m)-linux"
cargo build --locked --release -p tabletd -p tablet-center

stage="dist/tablet-center"
rm -rf "$stage"
mkdir -p "$stage/bin"
install -m 755 target/release/tabletd target/release/tablet-center "$stage/bin/"
cp -r packaging gnome-extension "$stage/"
cp install.sh uninstall.sh LICENSE README.md "$stage/"
tar -C dist -czf "dist/$name.tar.gz" tablet-center
echo "dist/$name.tar.gz"
