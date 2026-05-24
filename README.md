# `example-kvs`

[![license: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

> A [pulseengine.eu](https://pulseengine.eu) worked example: eclipse-score's
> `persistency::kvs` component treated end-to-end with the pulseengine stack
> — vendored upstream code + rivet typed artifacts + spar AADL + WIT binary
> contract + bazel + an artifact-driven verification gate that runs **real
> tests** against the **real upstream implementation**.
>
> **Not affiliated with the Eclipse Foundation.** The `vendor/rust_kvs/`
> directory contains the eclipse-score `rust_kvs` crate sources verbatim
> under Apache-2.0; see [`vendor/rust_kvs/ATTRIBUTION.md`](vendor/rust_kvs/ATTRIBUTION.md).

## What this is

Eclipse-score declares `persistency::kvs` as a sphinx-needs typed graph:
requirements, architecture, FMEA, design decisions, all as RST need
directives. The actual Rust implementation lives separately in
[eclipse-score/persistency](https://github.com/eclipse-score/persistency)
with no automated link between the spec and the code.

This repo:

1. Treats the same component through the pulseengine stack (rivet typed
   artifacts, spar AADL, WIT binary contract, witness MC/DC shape,
   sigil-signed manifest).
2. **Vendors the eclipse-score `rust_kvs` sources** under `vendor/rust_kvs/`
   so the verification gate runs against the real upstream implementation,
   not a toy stub.
3. **Wires `tools/verify.py` to actually run `bazel test`** per comp-req's
   `verified-by:` evidence list — there is no stubbing. Each artifact's
   bucket reflects what the real bazel test invocation reported.

## Finding: two upstream comp-reqs are unverified and unenforced

Running the gate against the vendored eclipse-score `rust_kvs` surfaces
a clean-room-verified gap. Two component requirements documented in
[`persistency/score/kvs/docs/requirements/index.rst`](https://github.com/eclipse-score/persistency/blob/main/score/kvs/docs/requirements/index.rst)
are accepted (`:status: valid`) but their declared behavior is not
implemented and not tested:

| Upstream comp-req | RST text (verbatim) | What the impl does |
|---|---|---|
| `comp_req__kvs__key_naming` | "shall accept keys that consist solely of alphanumeric characters, underscores, or dashes" | `Kvs::set_value("with space", _)` returns `Ok(())`. Same for keys containing `.`, `/`, or any other character. |
| `comp_req__kvs__key_length` | "shall limit the maximum length of a key to 32 bytes" | `Kvs::set_value(&"a".repeat(33), _)` returns `Ok(())`. No length check exists. |

Verified independently (clean-room search across both Rust and C++
codebases under eclipse-score): there is no `validate_key` / `check_key`
function anywhere, no length constant, no test case directive
(`.. test_case::`) tied to either comp-req, no documentation note saying
"validation happens at the IPC boundary." Both Rust `set_value` and the
C++ `set_value` accept any input.

**The gate calls this out by going RED.** Current `make verify` output:

```
BUCKET    ID                            EVIDENCE
─────────────────────────────────────────────────────────────────────
PASSED    COMP-REQ-KVS-KEY-ENCODING     1 test(s) all green
PASSED    COMP-REQ-KVS-VALUE-CHECKSUM   5 test(s) all green
PASSED    COMP-REQ-KVS-ATOMIC-STORE     5 test(s) all green
PASSED    COMP-REQ-KVS-INLINE-STORAGE   1 test(s) all green
FAILED    COMP-REQ-KVS-KEY-NAMING       3 test(s) failed:
                                          - test_..._space_rejected
                                          - test_..._dot_rejected
                                          - test_..._slash_rejected
FAILED    COMP-REQ-KVS-KEY-LENGTH       1 test(s) failed:
                                          - test_..._32_byte_cap_enforced
─────────────────────────────────────────────────────────────────────
4 PASSED, 2 FAILED, 0 MISSING
```

The surface tests at [`tests/surface/surface_tests.rs`](tests/surface/surface_tests.rs)
assert exactly what the upstream RST says — no pulseengine-side
embellishment. CI is intentionally red on this finding.

This is the **demonstration**: a coverage-percentage dashboard would
mark these comp-reqs "covered" because some test mentions the module
they belong to. Pulseengine's `verified-by:` mechanism forces
test-to-requirement traceability at the level of named test functions,
and the gate then asserts each named test PASSED. Requirements with no
verifying test stay loudly red.

## What this gate measures

Each comp-req artifact carries a `verified-by:` list of
`<bazel-target>:<test-fn-name>` entries. The gate:

1. **Discovers** — for each entry, checks the test exists in the
   target binary. Missing → bucket `MISSING`.
2. **Runs** — invokes `bazel test --test_arg=--exact --test_arg=<name>`
   per entry. Any non-zero exit → bucket `FAILED`. All green → bucket
   `PASSED`.

Each comp-req artifact lists its evidence. For the KEY-NAMING / KEY-LENGTH
falsifications, the response options are explicit:

- Add runtime validation to the impl → tests go green.
- Edit the upstream RST to drop those requirements → artifact updates,
  tests can be deleted.
- Downgrade the comp-req's `status` from `approved` to `draft` (the
  gate only enforces approved artifacts).

All three are legitimate engineering responses to evidence — the gate
makes the choice explicit, where a coverage-wedge visualisation makes
it implicit.

## Eclipse layer comparison

| Layer | What it adds | Eclipse equivalent |
|---|---|---|
| `artifacts/` (rivet typed YAML) | Validated typed graph: 8 reqs + 8 architecture + 2 FMEA + 2 decisions + 3 test-specs, schema-checked by `rivet validate`. Each comp-req carries a `verified-by:` evidence list. | sphinx-needs `comp_req` / `feat` / `feat_saf_fmea` / `dec_rec` / `testcase` directives |
| `vendor/rust_kvs/` (vendored upstream) | The actual eclipse-score KVS code, compiled as a bazel `rust_library` with all 248 upstream unit tests runnable via `bazel test //vendor/rust_kvs:rust_kvs_test` | Same code in eclipse-score/persistency, tested by its own CI |
| `tests/surface/surface_tests.rs` | One `#[test]` per comp-req, asserting requirement-as-test against the vendored upstream code (verbatim spec text, no embellishment) | None — eclipse tests are organized by module, not requirement |
| `arch/kvs.aadl` (spar AADL) | Typed feature group + subprogram signatures + ARP4761 safety properties | None |
| `arch/kvs.wit` (binary contract) | WIT interface that wit-bindgen turns into a Rust trait the impl must satisfy at link time | None — interface stops at the rendered diagram |
| `tools/verify.py` (artifact-driven gate) | Walks every approved comp-req, reads `verified-by:`, runs `bazel test` per entry, exits red on any gap | None — eclipse renders a coverage pie chart |
| `attestation/release-manifest.yaml` (sigil) | Signed in-toto-style attestation tying artifact hashes + WIT hash + evidence hashes | Green CI badge |

## What's *real* infrastructure vs *example skeleton*

| Layer | State |
|---|---|
| `rivet validate` on `artifacts/*.yaml` | ✅ **Builds + passes** (with 10 schema warnings about lifecycle completeness) |
| `vendor/rust_kvs/` upstream sources + tests | ✅ **244 of 248 tests pass natively**; 4 ignored. `bazel test //vendor/rust_kvs:rust_kvs_test` runs all of them. |
| `tests/surface/surface_tests.rs` comp-req gate | ✅ **Runs in bazel**; reports 6 PASSED + 4 FAILED. The 4 failures are confirmed-real spec falsifications (see "Finding" above). |
| `tools/verify.py` artifact-driven gate | ✅ **Shells out to bazel test per artifact**; reports 4 PASSED + 2 FAILED comp-reqs. |
| `bazel build //...` — AADL → WIT → wit-bindgen → Rust → .wasm component | ✅ **Builds + passes** locally; CI builds it on every push |
| `vendor/score_log_shim/` no-op stand-in for `score_log` | ✅ **Compiles + lets vendored rust_kvs tests run** without pulling baselibs_rust |
| `make aadl` / `make wit` via `spar` | ⚙️ Optional — requires `spar` installed; skips cleanly if missing |
| `verification/mc_dc_harness.rs` witness annotations | 📄 **Skeleton** showing what witness-instrumented tests look like; not wired into a witness build yet |
| `attestation/release-manifest.yaml` sigil-shape | 📄 **Skeleton** showing the manifest shape; `make attest` skips cleanly if sigil missing |

## Layout

```
example-kvs/
├── rivet.yaml                       # rivet project: common + score schemas
├── artifacts/
│   ├── requirements.yaml            # 10 reqs (4 stkh/feat, 6 comp);
│   │                                # each comp-req carries `verified-by:` evidence list
│   ├── architecture.yaml            # feat → comp → interface + ops + dd-sta + sw-units
│   └── safety-and-decisions.yaml    # 2 FMEA + 2 ADRs + 3 test-specs
├── arch/
│   ├── kvs.aadl                     # spar AADL package, ARP4761 properties
│   └── kvs.wit                      # WIT contract emitted from the AADL
├── src/lib.rs                       # WASM-component impl of arch/kvs.wit
├── vendor/
│   ├── rust_kvs/                    # eclipse-score rust_kvs sources (Apache-2.0)
│   │   ├── ATTRIBUTION.md           # source + license details
│   │   ├── LICENSE / NOTICE         # upstream
│   │   └── src/*.rs                 # verbatim copies (2-line Debug additions only)
│   ├── score_log_shim/              # no-op stand-in for baselibs_rust score_log
│   │   ├── README.md
│   │   ├── score_log/               # rlib: macros + ScoreDebug trait
│   │   └── score_log_derive/        # proc-macro for #[derive(ScoreDebug)]
│   └── rivet-schemas/               # pinned snapshot of rivet's schema set
├── tests/surface/
│   ├── BUILD.bazel
│   └── surface_tests.rs             # one #[test] per comp-req artifact
├── verification/
│   ├── README.md                    # witness MC/DC explainer
│   └── mc_dc_harness.rs             # skeleton showing witness annotations
├── attestation/
│   └── release-manifest.yaml        # sigil-shaped signed release manifest
├── tools/
│   └── verify.py                    # artifact-driven verification gate
├── Makefile                         # validate / aadl / wit / verify / attest / bazel
├── BUILD.bazel / MODULE.bazel       # bazel module + WASM-component build
├── Cargo.toml / Cargo.lock          # workspace (vendored crates + crate_universe shim)
├── LICENSE                          # Apache-2.0 (this repo)
└── README                           # you are here
```

## Run it

```sh
make validate    # rivet validate against the typed schema
make verify      # artifact-driven verification gate (needs bazel installed)
make bazel       # bazel build + bazel test //... (the WASM chain)
make aadl        # spar validates the AADL package (requires spar)
make wit         # spar AADL → WIT round-trip check (requires spar)
make attest      # sigil-signed release manifest (requires sigil)
```

Test the vendored upstream crate directly with cargo:

```sh
cargo test --workspace        # 244 tests pass, 4 ignored, 0 failed
```

Run only the surface tests (one per comp-req):

```sh
bazel test //tests/surface:surface_tests
# 6 PASSED, 4 FAILED — the 4 are the upstream-spec falsifications.
```

## What this is *not*

- **Not an automatic translation of eclipse-score content.** Every
  artifact YAML here was hand-authored from the eclipse equivalents.
  An automated `score → pulseengine` converter is a separate
  workstream — see the
  [playground-eclipse-score](https://github.com/pulseengine/playground-eclipse-score)
  workspace for that.
- **Not endorsed by the Eclipse Foundation or eclipse-score
  maintainers.** This is a pulseengine demonstration that vendors
  upstream Apache-2.0 sources; issues belong here, eclipse-score's
  repo is unchanged.
- **Not a finding-and-tell exercise.** The KEY-NAMING / KEY-LENGTH
  falsification is a legitimate demonstrable gap in the upstream
  implementation, but the upstream project is itself early-stage —
  the comp-reqs probably haven't reached a "MUST be implemented by
  release X" gate yet. The demo's value is the **methodology**: an
  artifact-driven gate that turns those gaps into CI signal rather
  than an open ticket nobody runs against the code.
- **Not certification-ready.** rivet, spar, witness, and sigil are
  all pre-1.0; the verify gate exercises them against real content
  but no part has been independently TCL-assessed.

## Cross-references

- The conversion playground: https://github.com/pulseengine/playground-eclipse-score
- Rivet (typed traceability): https://github.com/pulseengine/rivet
- Spar (AADL → WIT): https://github.com/pulseengine/spar
- `rules_wasm_component` (WIT → WASM): https://github.com/pulseengine/rules_wasm_component
- Witness (MC/DC for Wasm): https://github.com/pulseengine/witness
- Sigil (signed attestation): https://github.com/pulseengine/sigil

## License

This repo: Apache-2.0. See [LICENSE](LICENSE).
Vendored eclipse-score code under `vendor/rust_kvs/`: Apache-2.0, see
`vendor/rust_kvs/LICENSE` and `vendor/rust_kvs/NOTICE`.
