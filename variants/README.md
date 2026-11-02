# `variants/`

Per-deployment-context configuration for the example-kvs build.

Two pieces:

- [`feature-model.yaml`](feature-model.yaml) — declares the variant
  axes (a single `deployment-context: dev | prod` choice for this
  example).
- [`bindings.yaml`](bindings.yaml) — per-variant mapping to:
  - **`kvs-config:`** — KvsBuilder runtime dials
    (`KvsDefaults`, `KvsLoad`, `snapshot_max_count`)
  - **`bazel-config:`** — bazel `--config=` profile from
    [`/.bazelrc`](../.bazelrc), which controls the compile-time
    safety-relevant rustc flags (`lto`, `codegen-units`, `panic`,
    `overflow-checks`, `strip`)
  - **`out-of-scope-for:`** — comp-req IDs the verify gate skips
    for this variant

## Why a variant model is needed here

A "variant" here is three axes bound together by name:

### 1. Runtime builder dials

Upstream eclipse-score's `rust_kvs` has **no cargo features** in the
core library. All deployment-context behavior is configured at the
*builder* — `KvsBuilder::new(InstanceId(...))` accepts
`defaults: KvsDefaults`, `kvs_load: KvsLoad`, and the backend
exposes `snapshot_max_count`. These dials are runtime.

A `dev` build is allowed to use `KvsDefaults::Ignored` and a
1-deep snapshot ring; a `prod` build mandates `Required` for both
and a 10-deep ring. Same source, same binary code, different
construction.

### 2. Compile-time rustc profile

For safety-critical Rust builds the compile profile is *not*
neutral — assessors typically expect `lto=fat`,
`codegen-units=1`, `panic=abort`, `overflow-checks=on`,
`strip=symbols`. The `prod` variant binds to **two** related
configs in [`/.bazelrc`](../.bazelrc):

- **`--config=prod`** — used for `bazel test` and `make verify`.
  Applies lto=fat + codegen-units=1 + overflow-checks=on +
  strip=symbols + embed-bitcode=yes. **Omits panic=abort**:
  Rust's `#[test]` harness needs unwinding to report failures
  (the alternative `-Zpanic_abort_tests` is nightly-only).
- **`--config=prod_ship`** — extends `prod` with `panic=abort`.
  Use for `bazel build` of the actually-shipped binary, never
  for `bazel test`:

  ```sh
  bazel build --config=prod_ship //:kvs_component
  ```

The split exists because the deployed ASIL-B binary should not
carry unwinding machinery (smaller, no cleanup-path semantics
for an assessor to argue about), but the test binary must.

The `dev` variant uses `--config=dev` →
`--compilation_mode=fastbuild` (debug asserts on, no
optimization, fast incremental rebuilds).

### 3. Audit scope (this fork's invention)

The audit-scope axis is unique to this fork. It lets a `dev` build
honestly exempt comp-reqs like `COMP-REQ-KVS-INLINE-STORAGE` (which
the upstream impl cannot honor without witness allocator
instrumentation) without rewriting the artifact YAMLs.

## Running per-variant

```sh
make verify VARIANT=dev      # default — comp-reqs scoped to dev variant
make verify VARIANT=prod     # full comp-req set + prod bazel profile
make bazel  VARIANT=prod     # bazel build/test under --config=prod
```

The variant flows through to:

1. **`tools/verify.py --variant <name>`** — filters comp-reqs by the
   `out-of-scope-for:` list in `bindings.yaml` before driving
   `bazel test` per `verified-by:` entry.
2. **bazel `--config=$(VARIANT)`** — picks up the rustc safety flags
   from `/.bazelrc` (compilation_mode + lto + codegen-units + panic
   + overflow-checks + strip for `prod`; fastbuild for `dev`).

The **`kvs-config:` block** in `bindings.yaml` is a *documented
binding*, not a test-injection. The surface tests use their own
per-test `KvsBuilder` (with their own tempdirs) so they don't
depend on the variant's runtime dials. `kvs-config:` records the
contract a real deployment would use — auditable, machine-readable,
not load-bearing for the gate output.

## Adding a new variant

1. Add the value to `feature-model.yaml`'s `values:` list.
2. Add a `binding:` entry in `bindings.yaml` with `kvs-config:` and
   `out-of-scope-for:`.
3. If the variant needs surface-test changes (e.g. asserting strict
   key validation), add the per-variant arm in
   `tests/surface/surface_tests.rs`.

## What this is *not*

- **Not a cargo feature graph.** Upstream rust_kvs has no features,
  and this fork respects that — variants are declared in YAML and
  applied at runtime, not via `--features`.
- **Not a TCL-qualified variant management system.** Eclipse-score
  has `feat_req__persistency__variant_management` in its docs but
  no implementation; pulseengine's variant model here is the
  minimum-viable shape, not the answer.
