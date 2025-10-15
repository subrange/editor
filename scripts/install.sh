#!/usr/bin/env bash
set -euo pipefail

# Build everything as the current user (do NOT run this script with sudo)
# This avoids creating root-owned files under ./target which will break `cargo test`.
npm run build:native

# Binaries that should exist after the build   
# binaries=(
#   "src/rust-bf/target/release/bf"
#   "src/ripple-asm/target/release/rasm"
#   "src/ripple-asm/target/release/rlink"
#   "rbt/target/release/rbt"
#   "target/release/rvm"
#   "src/bf-macro-expander/target/release/bfm"
# )

# Verify artifacts exist
for bin in "${binaries[@]}"; do
  if [[ ! -x "$bin" ]]; then
    echo "error: expected built binary '$bin' not found or not executable" >&2
    exit 1
  fi
done

# Use sudo ONLY for the install step
# TODO: Don't force user to install.
# DEST="/usr/local/bin"
# echo "Installing to $DEST (you may be prompted for your password)…"
# sudo install -m 0755 "${binaries[@]}" "$DEST"

echo "Done."