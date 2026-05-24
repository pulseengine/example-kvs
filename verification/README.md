# Verification — witness MC/DC + artifact-driven gate

Two parts:

1. **`mc_dc_harness.rs`** — skeleton showing how `witness`
   (https://github.com/pulseengine/witness) instruments the KVS
   implementation under test and emits MC/DC truth tables per
   tested predicate. Eclipse-score has nothing equivalent: their
   `testcase` need carries `partially_verifies`/`fully_verifies`
   links but no MC/DC evidence, no truth tables, no gap-identification.

2. **`../tools/verify.py`** — the artifact-driven verification gate
   (see top-level README). Walks the rivet artifacts looking for
   `comp-req`s with `status: approved`, finds tests by name
   convention (`test_<id-lowercased-dashed>_*`), runs them, reports
   pass/fail/missing per artifact. Same shape as the LS-N gate in
   `pulseengine/meld`.

The two together would form the verification spine: witness gives
you the *evidence* per individual test (truth table + missing
condition vectors); `verify.py` gives you the *aggregate verdict*
per artifact (which comp-reqs are covered, which are not).
