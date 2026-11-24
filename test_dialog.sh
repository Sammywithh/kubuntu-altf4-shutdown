#!/usr/bin/env bash
# Build from source (or use release archive binary) and launch the Kubuntu Shutdown Dialog

set -euo pipefail

: "${HOME:?HOME ist nicht gesetzt}"
if [[ "${HOME}" != /* || "${HOME}" == "/" ]]; then
    echo "Fehler: Ungültiger HOME-Pfad: ${HOME}" >&2
    exit 1
fi

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ -f "${DIR}/Cargo.toml" ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Fehler: 'cargo' wurde nicht gefunden, obwohl Cargo.toml vorliegt." >&2
        exit 1
    fi
    cargo build --release --locked --manifest-path "${DIR}/Cargo.toml"
    BIN="${DIR}/target/release/kubuntu-shutdown-dialog"
elif [[ -x "${DIR}/kubuntu-shutdown-dialog" ]]; then
    BIN="${DIR}/kubuntu-shutdown-dialog"
else
    echo "Fehler: Weder Cargo.toml noch ausführbares kubuntu-shutdown-dialog Binary gefunden." >&2
    exit 1
fi

exec "${BIN}" "$@"
