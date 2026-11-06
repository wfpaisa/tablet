#!/bin/sh
# Builds the release binaries in Docker (Debian bookworm, so the archive
# runs on older glibc too) and packs them with the installer into
# dist/tablet-center-<arch>-linux.tar.gz plus its .sha256, the files attached
# to GitHub releases. The name has no version so the README can link to
# releases/latest/download/. Usage: scripts/package.sh
set -eu
cd "$(dirname "$0")/.."

command -v docker >/dev/null || { echo "package: docker not found" >&2; exit 1; }
name="tablet-center-$(uname -m)-linux"

# The container has no network: fetch the crates into the host registry first.
cargo fetch --locked --quiet
cargo_home=${CARGO_HOME:-$HOME/.cargo}
docker run --rm --network none --user "$(id -u):$(id -g)" -v "$PWD":/src -w /src \
	-v "$cargo_home/registry":/cargo/registry \
	-e HOME=/tmp -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/src/target/docker \
	rust:1-bookworm sh -c 'RUSTUP_TOOLCHAIN=$RUST_VERSION cargo build --release --locked --offline -p tabletd -p tablet-center'

stage="dist/tablet-center"
rm -rf dist
mkdir -p "$stage/bin"
install -m 755 target/docker/release/tabletd target/docker/release/tablet-center "$stage/bin/"
cp -r packaging gnome-extension "$stage/"
cp install.sh uninstall.sh LICENSE README.md "$stage/"
tar -C dist -czf "dist/$name.tar.gz" tablet-center
(cd dist && sha256sum "$name.tar.gz" >"$name.tar.gz.sha256")
echo "dist/$name.tar.gz (needs $(objdump -T "$stage/bin/tablet-center" | grep -oE 'GLIBC_[0-9.]+' | sort -Vu | tail -1))"
