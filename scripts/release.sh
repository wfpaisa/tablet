#!/bin/sh
# Starts a release: sets the version, commits, tags and pushes. GitHub
# Actions (release.yml) then builds the archive and publishes the release.
# Usage: scripts/release.sh 0.2.0
set -eu
cd "$(dirname "$0")/.."

version=${1:-}
current=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
if ! printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
	echo "Usage: scripts/release.sh X.Y.Z (now $current)" >&2
	exit 1
fi
fail() {
	echo "release: $*" >&2
	exit 1
}
[ "$version" != "$current" ] || fail "the version is already $version"
[ "$(git branch --show-current)" = main ] || fail "switch to main first"
[ -z "$(git status --porcelain)" ] || fail "commit or stash your changes first"
! git rev-parse -q --verify "refs/tags/v$version" >/dev/null || fail "tag v$version already exists"
git fetch -q origin main
# Local commits are fine (they go up with the tag); missing ones are not.
git merge-base --is-ancestor origin/main HEAD || fail "pull first: origin/main has commits main lacks"

cargo test --locked --workspace

# Only the workspace version, the first version line in Cargo.toml.
sed -i "0,/^version = \"$current\"/s//version = \"$version\"/" Cargo.toml
cargo update -q --workspace
git commit -q -m "Release $version" Cargo.toml Cargo.lock
git tag -a "v$version" -m "Tablet Center $version"
git push -q --atomic origin main "v$version"
echo "Pushed v$version. The release appears once the workflow finishes:"
echo "https://github.com/wfpaisa/tablet/actions/workflows/release.yml"
