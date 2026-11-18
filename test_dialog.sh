#!/usr/bin/env bash
# Build (if needed) and launch the Rust Kubuntu Shutdown Dialog directly
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="${DIR}/target/release/kubuntu-shutdown-dialog"

if [ ! -x "${BIN}" ]; then
    cargo build --release --manifest-path "${DIR}/Cargo.toml"
fi

exec "${BIN}" "$@"
