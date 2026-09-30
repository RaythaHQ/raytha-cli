#!/bin/sh
# Installs the `raytha` CLI: downloads the release for this OS/CPU, verifies its SHA-256, and puts
# the binary in a directory on your PATH.
#
#   curl -fsSL <url-of-this-script> | sh
#   curl -fsSL <url-of-this-script> | sh -s -- --version v0.1.0
#
# Environment:
#   RAYTHA_VERSION       release tag to install (default: latest), same as --version
#   RAYTHA_INSTALL_DIR   where to put the binary (default: $HOME/.local/bin)
#   RAYTHA_INSTALL_BASE  where releases are hosted. Change this one line (or set the variable) if
#                        the binaries move. Layout expected below it:
#                          <base>/latest/download/<asset>
#                          <base>/download/<tag>/<asset>
#                        with assets raytha-<target>.tar.gz and SHA256SUMS.
set -eu

BASE="${RAYTHA_INSTALL_BASE:-https://github.com/RaythaHQ/raytha-cli/releases}"
VERSION="${RAYTHA_VERSION:-latest}"
INSTALL_DIR="${RAYTHA_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*" >&2; }
die() { say "error: $*"; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --version) [ $# -ge 2 ] || die "--version needs a value"; VERSION="$2"; shift 2 ;;
    --version=*) VERSION="${1#--version=}"; shift ;;
    --dir) [ $# -ge 2 ] || die "--dir needs a value"; INSTALL_DIR="$2"; shift 2 ;;
    --dir=*) INSTALL_DIR="${1#--dir=}"; shift ;;
    -h|--help) sed -n '2,17p' "$0" 2>/dev/null || true; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
done

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Linux) os_part="unknown-linux-musl" ;;
  Darwin) os_part="apple-darwin" ;;
  *) die "unsupported OS '$os'. On Windows use install.ps1." ;;
esac
case "$arch" in
  x86_64|amd64) arch_part="x86_64" ;;
  aarch64|arm64) arch_part="aarch64" ;;
  *) die "unsupported CPU '$arch'" ;;
esac
target="${arch_part}-${os_part}"
asset="raytha-${target}.tar.gz"

if [ "$VERSION" = "latest" ]; then
  root="$BASE/latest/download"
else
  root="$BASE/download/$VERSION"
fi

if command -v curl >/dev/null 2>&1; then
  fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
  fetch() { wget -q -O "$2" "$1"; }
else
  die "need curl or wget"
fi

if command -v sha256sum >/dev/null 2>&1; then
  sha256() { sha256sum "$1" | cut -d ' ' -f 1; }
elif command -v shasum >/dev/null 2>&1; then
  sha256() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
else
  die "need sha256sum or shasum to verify the download"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

say "Downloading $asset ($VERSION)"
fetch "$root/$asset" "$tmp/$asset" || die "could not download $root/$asset"
fetch "$root/SHA256SUMS" "$tmp/SHA256SUMS" || die "could not download $root/SHA256SUMS"

expected="$(grep "  $asset\$" "$tmp/SHA256SUMS" | cut -d ' ' -f 1 || true)"
[ -n "$expected" ] || die "$asset is not listed in SHA256SUMS"
actual="$(sha256 "$tmp/$asset")"
[ "$expected" = "$actual" ] || die "checksum mismatch for $asset (expected $expected, got $actual)"

tar -xzf "$tmp/$asset" -C "$tmp"
[ -f "$tmp/raytha" ] || die "archive did not contain a raytha binary"

mkdir -p "$INSTALL_DIR"
if command -v install >/dev/null 2>&1; then
  install -m 0755 "$tmp/raytha" "$INSTALL_DIR/raytha"
else
  cp "$tmp/raytha" "$INSTALL_DIR/raytha"
  chmod 0755 "$INSTALL_DIR/raytha"
fi

say "Installed $("$INSTALL_DIR/raytha" --version 2>/dev/null || echo raytha) to $INSTALL_DIR/raytha"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) say "Add it to your PATH:  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
say "Next: set RAYTHA_URL and RAYTHA_API_KEY, then run 'raytha doctor' and 'raytha guide'."
