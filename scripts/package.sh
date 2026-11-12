#!/usr/bin/env bash
# Builds the portable release archive and packs it into
# dist/tablet-center-<arch>-linux.tar.gz plus its .sha256.
# If Docker or Podman is available, it can build in a clean Debian container
# for maximum glibc portability; otherwise (or with --local) it packages
# the local release build.
set -euo pipefail
cd "$(dirname "$0")/.."

name="tablet-center-$(uname -m)-linux"
stage="dist/tablet-center"
bin_dir="target/release"

use_container=false
container_cmd=""

if [[ "${1:-}" != "--local" ]]; then
    if command -v docker >/dev/null 2>&1; then
        use_container=true
        container_cmd="docker"
    elif command -v podman >/dev/null 2>&1; then
        use_container=true
        container_cmd="podman"
    fi
fi

if $use_container; then
    echo "Building in $container_cmd (Debian bookworm) for portable glibc compatibility..."
    cargo fetch --locked --quiet
    cargo_home=${CARGO_HOME:-$HOME/.cargo}
    $container_cmd run --rm --network none --user "$(id -u):$(id -g)" -v "$PWD":/src -w /src \
        -v "$cargo_home/registry":/cargo/registry \
        -e HOME=/tmp -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/src/target/docker \
        rust:1-bookworm sh -c 'RUSTUP_TOOLCHAIN=$RUST_VERSION cargo build --release --locked --offline -p tabletd -p tablet-center'
    bin_dir="target/docker/release"
else
    echo "Packaging from local release build..."
    if [[ ! -f target/release/tabletd || ! -f target/release/tablet-center ]]; then
        cargo build --release -p tabletd -p tablet-center
    fi
fi

mkdir -p "$stage/bin"
install -m 755 "$bin_dir/tabletd" "$bin_dir/tablet-center" "$stage/bin/"
cp -r packaging gnome-extension "$stage/"
cp install.sh uninstall.sh LICENSE README.md "$stage/"
tar -C dist -czf "dist/$name.tar.gz" tablet-center
(cd dist && sha256sum "$name.tar.gz" >"$name.tar.gz.sha256")
echo "Created portable archive: dist/$name.tar.gz"
