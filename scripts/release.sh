#!/usr/bin/env bash
# Publishes a new Tablet Center release on GitHub, all from this machine.
#
#   scripts/release.sh            bump the patch version (1.3.1 -> 1.3.2)
#   scripts/release.sh 1.4.0      release that version
#
# Bumps the version in Cargo.toml, runs `just check` and builds the portable
# archive with `just package` (needs Docker). Then, after asking, commits,
# tags, pushes and creates the GitHub release with `gh`. If anything fails
# before the commit, Cargo.toml and Cargo.lock are restored.
set -euo pipefail

REPO="wfpaisa/tablet"
ASSET="dist/tablet-center-$(uname -m)-linux.tar.gz"

info() { printf '\033[1;34m::\033[0m %s\n' "$*"; }
die() { printf '\033[1;31mxx\033[0m %s\n' "$*" >&2; exit 1; }

cd "$(dirname "$0")/.."

for tool in git gh just docker cargo; do
    command -v "$tool" >/dev/null || die "Missing $tool"
done
gh auth status >/dev/null 2>&1 || die "Not logged in to GitHub: run gh auth login"

[[ "$(git branch --show-current)" == main ]] || die "Not on main"
[[ -z "$(git status --porcelain)" ]] || die "Uncommitted changes: commit or stash them first"
git fetch --quiet origin main
# Local commits are fine (they go up with the release); missing ones are not.
git merge-base --is-ancestor origin/main HEAD || die "origin/main has commits main lacks: pull first"

current="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
[[ -n "$current" ]] || die "No version in Cargo.toml"
if [[ $# -ge 1 ]]; then
    version="${1#v}"
else
    IFS=. read -r major minor patch <<<"$current"
    version="${major}.${minor}.$((patch + 1))"
fi
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Invalid version: $version"
tag="v${version}"
[[ "$version" != "$current" ]] || die "Cargo.toml is already at $version"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && die "Tag $tag already exists"
gh release view "$tag" --repo "$REPO" >/dev/null 2>&1 && die "Release $tag already exists"

info "Releasing $tag (now $current)"
# Until the commit, a failure leaves the tree as it was.
trap 'git checkout --quiet -- Cargo.toml Cargo.lock' EXIT
# Only the workspace version, the first version line in Cargo.toml.
sed -i "0,/^version = \"$current\"/s//version = \"$version\"/" Cargo.toml
cargo update --workspace --offline --quiet

info "Running just check"
just check

info "Building the portable archive (Docker)"
just package
[[ -f "$ASSET" && -f "$ASSET.sha256" ]] || die "just package did not produce $ASSET"

read -rp "Commit, tag, push and publish $tag? [y/N] " answer
[[ "$answer" == [yY]* ]] || die "Cancelled; nothing was published"

info "Committing, tagging and pushing"
git commit --quiet -m "Release $tag" Cargo.toml Cargo.lock
trap - EXIT
git tag -a "$tag" -m "Tablet Center $version"
git push --quiet --atomic origin main "$tag"

info "Creating the GitHub release"
gh release create "$tag" "$ASSET" "$ASSET.sha256" \
    --repo "$REPO" --verify-tag --title "Tablet Center $tag" --generate-notes

info "Done: https://github.com/$REPO/releases/tag/$tag"
