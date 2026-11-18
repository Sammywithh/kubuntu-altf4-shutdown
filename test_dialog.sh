#!/usr/bin/env bash
# Build (if needed) and launch the Rust Kubuntu Shutdown Dialog directly
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -x "${DIR}/target/release/kubuntu-shutdown-dialog" ]; then
    BIN="${DIR}/target/release/kubuntu-shutdown-dialog"
elif [ -x "${DIR}/kubuntu-shutdown-dialog" ]; then
    BIN="${DIR}/kubuntu-shutdown-dialog"
else
    cargo build --release --manifest-path "${DIR}/Cargo.toml"
    BIN="${DIR}/target/release/kubuntu-shutdown-dialog"
fi

exec "${BIN}" "$@"
