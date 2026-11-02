# Vendored from eclipse-score/persistency

This directory is a **verbatim source copy** of the `rust_kvs` crate from
the Eclipse S-CORE persistency repository:

  - **Upstream:** https://github.com/eclipse-score/persistency
  - **Path:** `src/rust/rust_kvs/`
  - **License:** Apache License 2.0 (see [`LICENSE`](LICENSE))
  - **Copyright:** Contributors to the Eclipse Foundation (see [`NOTICE`](NOTICE))
  - **Snapshot date:** 2026-05-24

## Why vendored, not depended-on

This example uses Bazel + `rules_wasm_component` + `rules_rust`, with
crate dependencies routed through `crate_universe`. The upstream
crate depends on `score_log` from `eclipse-score/baselibs_rust`, which
is a git-only dependency that crate_universe cannot resolve cleanly
without pulling in the entire baselibs_rust workspace.

Rather than vendor *all* of baselibs_rust, this repo:

1. Vendors **only** the `rust_kvs` crate sources here.
2. Provides a tiny `score_log_shim/` crate (this repo's `vendor/`)
   that re-exports compatible macros + a `ScoreDebug` trait. The
   shim is a stand-in; it preserves the `rust_kvs` source verbatim.
3. Wires both into a `rust_library` Bazel target.

## Local modifications

**None to the Rust source files.** Every `.rs` file in `src/` matches
the upstream commit at snapshot date.

The only non-upstream file in this directory is the BUILD target,
which lives outside this folder (`/BUILD.bazel`) and references the
sources by relative path.

## How to refresh

```sh
# From example-kvs root:
SRC=/path/to/eclipse-score-fork/.rivet/repos/persistency/src/rust/rust_kvs
cp "$SRC"/src/*.rs vendor/rust_kvs/src/
cp "$SRC"/../../../LICENSE vendor/rust_kvs/LICENSE
cp "$SRC"/../../../NOTICE  vendor/rust_kvs/NOTICE
# Update the snapshot date in this file.
```
