#!/usr/bin/env bash
# miri.sh — per-module miri UB-check over the vendored rust_kvs crate.
#
# Matches the intent of eclipse-score's BUILD:36-105 miri_test targets
# (one per module: error_code, json_backend, kvs, kvs_api, kvs_builder,
# kvs_mock, kvs_serialize, kvs_value). That rule is in a custom
# rules_rust fork; this script gives us the same coverage without
# the fork.
#
# Usage:
#   tools/miri.sh                # run all modules
#   tools/miri.sh kvs_builder    # narrow to one module
#
# Requirements:
#   rustup toolchain install nightly
#   rustup +nightly component add miri

set -euo pipefail

MODULES=(
    error_code
    json_backend
    kvs
    kvs_api
    kvs_builder
    kvs_mock
    kvs_serialize
    kvs_value
)

if [[ $# -gt 0 ]]; then
    MODULES=("$1")
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not found — install rustup" >&2
    exit 1
fi
if ! cargo +nightly miri --version >/dev/null 2>&1; then
    echo "cargo +nightly miri not found; install with:" >&2
    echo "  rustup toolchain install nightly" >&2
    echo "  rustup +nightly component add miri" >&2
    exit 1
fi

failures=0
for module in "${MODULES[@]}"; do
    echo "── miri: $module ──"
    if cargo +nightly miri test --manifest-path vendor/rust_kvs/Cargo.toml \
            -- "${module}::" --test-threads=1; then
        echo "PASSED $module"
    else
        echo "FAILED $module"
        failures=$((failures + 1))
    fi
done

if [[ $failures -gt 0 ]]; then
    echo "── $failures of ${#MODULES[@]} modules failed under miri ──" >&2
    exit 1
fi
echo "── all ${#MODULES[@]} modules clean under miri ──"
