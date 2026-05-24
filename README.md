# `example-kvs`

[![license: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

> A [pulseengine.eu](https://pulseengine.eu) worked example: eclipse-score's
> `persistency::kvs` component treated end-to-end with the pulseengine stack
> — rivet typed artifacts + spar AADL + WIT binary contract +
> witness MC/DC harness + sigil-signed release manifest + artifact-driven
> verification gate.
>
> **Not affiliated with the Eclipse Foundation.** Derived from
> [eclipse-score/persistency](https://github.com/eclipse-score/persistency)
> as a worked example. The eclipse-score equivalents of each artifact
> are named in their descriptions.

## What this is

Eclipse-score declares `persistency::kvs` as a sphinx-needs typed
graph: requirements, architecture, FMEA, design decisions, all as
RST need directives. The actual Rust implementation lives separately
with no automated link to the spec.

This repo shows what the same component looks like when expressed
through the full pulseengine stack. Each layer adds something
eclipse's setup doesn't have:

| Layer | What it adds | Eclipse equivalent |
|---|---|---|
| `artifacts/` (rivet typed YAML) | Validated typed graph: 8 requirements + 8 architecture elements + 2 FMEA + 2 decisions + 3 test-specs, all schema-checked by `rivet validate` | sphinx-needs `comp_req` / `feat` / `feat_saf_fmea` / `dec_rec` / `testcase` directives |
| `arch/kvs.aadl` (spar AADL) | Typed feature group + subprogram signatures + ARP4761 safety properties | None — eclipse has no architecture-model file |
| `arch/kvs.wit` (binary contract) | WIT interface that wit-bindgen turns into a Rust trait the impl must satisfy at link time | None — interface stops at the rendered diagram |
| `verification/mc_dc_harness.rs` (witness) | Truth-table evidence per tested predicate, with explicit gap-identification for masking-MC/DC | `testcase` need carries pass/fail only |
| `tools/verify.py` (artifact-driven gate) | Walks the artifact list, finds tests by name convention, reports PASSED / FAILED / **MISSING** per artifact, exits red on any gap | None — eclipse renders a coverage pie chart on the docs site; missing tests are a wedge in the pie, not a CI gate |
| `attestation/release-manifest.yaml` (sigil) | Signed in-toto-style attestation tying artifact hashes + WIT contract hash + evidence hashes to a release | Green CI badge ("`bazel run //:docs_check` succeeded") |

## What's *real* infrastructure vs *example skeleton*

| Layer | State |
|---|---|
| `rivet validate` on `artifacts/*.yaml` | ✅ **Builds + passes** locally and in CI |
| `tools/verify.py` artifact-driven gate | ✅ **Builds + passes** locally and in CI |
| `bazel build //...` — AADL → WIT → wit-bindgen → Rust → .wasm component | ✅ **Builds + passes** locally (`bazel test //:kvs_component_test` green); CI builds it on every push |
| `make aadl` / `make wit` via `spar` | ⚙️ Optional — requires `spar` installed; skips cleanly if missing |
| `verification/mc_dc_harness.rs` witness annotations | 📄 **Skeleton** showing what witness-instrumented tests look like; not wired into a witness build yet |
| `attestation/release-manifest.yaml` sigil-shape | 📄 **Skeleton** showing the manifest shape; `make attest` skips cleanly if sigil missing |

The headline: **the AADL → WIT → wit-bindgen → Rust → .wasm
component chain actually builds**. `src/lib.rs` implements the
`Guest` trait that wit-bindgen generated from `arch/kvs.wit`; if
a signature drifts, the Rust compile fails. The binary contract
is real, not aspirational. This is the central operational
difference from eclipse-score's interface-as-documentation model.

## Layout

```
example-kvs/
├── rivet.yaml                     # rivet project: common + score schemas
├── artifacts/
│   ├── requirements.yaml          # 8 reqs across stkh / feat / comp levels
│   ├── architecture.yaml          # feat → comp → interface + 5 ops + dd-sta + sw-units
│   └── safety-and-decisions.yaml  # 2 FMEA entries + 2 ADRs + 3 test-specs
├── arch/
│   ├── kvs.aadl                   # spar AADL package, ARP4761 properties
│   └── kvs.wit                    # WIT contract emitted from the AADL
├── verification/
│   ├── README.md                  # witness MC/DC explainer
│   └── mc_dc_harness.rs           # skeleton showing witness annotations
├── attestation/
│   └── release-manifest.yaml      # sigil-shaped signed release manifest
├── tools/
│   └── verify.py                  # artifact-driven verification gate
├── vendor/
│   └── rivet-schemas/             # pinned snapshot of rivet's schema set
├── Makefile                       # validate / aadl / wit / verify / attest
├── LICENSE                        # Apache-2.0
└── README                         # you are here
```

## Run it

```sh
make validate    # rivet validate against the typed schema (always works)
make verify      # artifact-driven verification gate (always works)
make aadl        # spar validates the AADL package (requires spar)
make wit         # spar AADL → WIT round-trip check (requires spar)
make attest      # sigil-signed release manifest (requires sigil)
```

Today's `make verify` output:

```
PASSED    COMP-REQ-KVS-KEY-NAMING                       1 test(s) all green
PASSED    COMP-REQ-KVS-VALUE-CHECKSUM                   1 test(s) all green
PASSED    COMP-REQ-KVS-ATOMIC-STORE                     1 test(s) all green
MISSING   COMP-REQ-KVS-INLINE-STORAGE                   no test matching `test_comp_req_kvs_inline_storage_*`
3 PASSED, 0 FAILED, 1 MISSING
```

The MISSING is intentional — `COMP-REQ-KVS-INLINE-STORAGE` is
approved but has no harness coverage. Run `make verify` after
adding a `test_comp_req_kvs_inline_storage_*` to
`verification/mc_dc_harness.rs` and the bucket clears.

This is the operational difference the example illustrates:
**eclipse-score's coverage report tells you about gaps**;
**pulseengine's verify gate makes gaps fail CI**.

## What this is *not*

- **Not a complete persistency::kvs implementation.** The Rust
  source files referenced in the artifacts (`src/key_validator.rs`
  etc.) are not in this repo — they'd live in a separate crate
  that imports the WIT contract.
- **Not an automatic translation of eclipse-score content.** Every
  artifact here was hand-authored from the eclipse equivalents.
  An automated `score → pulseengine` converter is a separate
  workstream — see the
  [playground-eclipse-score](https://github.com/pulseengine/playground-eclipse-score)
  workspace for that.
- **Not endorsed by the Eclipse Foundation or eclipse-score
  maintainers.** This is a pulseengine demonstration, not a
  collaboration. Issues belong here; eclipse-score's RST is
  unchanged.
- **Not certification-ready.** rivet, spar, witness, and sigil
  are all pre-1.0; this example exercises them against real
  content but no part has been independently TCL-assessed. See
  each tool's `SAFETY.md` for self-assessed scope.

## Cross-references

- The conversion playground: https://github.com/pulseengine/playground-eclipse-score
- Rivet (typed traceability): https://github.com/pulseengine/rivet
- Spar (AADL → WIT): https://github.com/pulseengine/spar
- `rules_wasm_component` (WIT → WASM): https://github.com/pulseengine/rules_wasm_component
- Witness (MC/DC for Wasm): https://github.com/pulseengine/witness
- Sigil (signed attestation): https://github.com/pulseengine/sigil

## License

Apache-2.0. See [LICENSE](LICENSE).
