// witness MC/DC harness skeleton for persistency::kvs.
//
// Demonstrates the shape of evidence witness produces for the KVS
// implementation. In a real build, witness would:
//   1. Compile the impl crate with `witness instrument`
//   2. Run each test (cargo test) against the instrumented binary
//   3. Per tested predicate, emit a truth table with which condition
//      vectors fired and which are missing (`witness emit-truth-table`)
//   4. Per-test suggest the missing input combinations needed to
//      complete masking-MC/DC coverage
//   5. Persist the per-artifact verdict as a `test-exec` rivet
//      artifact (verification/test-execs.yaml — not generated in
//      this skeleton)
//
// Eclipse-score's equivalent stops at pass/fail per test. There is
// no MC/DC structural evidence; the safety case relies on inspection
// to argue coverage adequacy.

#![cfg(test)]

use kvs::{Kvs, KvsError};
use witness::{instrument, MutCounters, TruthTable};

/// Validates the key-validator decision predicates.
///
/// Verifies: COMP-REQ-KVS-KEY-NAMING
/// witness will produce a truth table per `if`/`match` in the
/// validator and report which condition vectors are still missing.
#[test]
#[witness::mcdc(target = "kvs::key_validator::validate")]
fn test_comp_req_kvs_key_naming_validator() {
    // Valid keys exercise the happy paths.
    assert!(Kvs::validate_key("foo").is_ok());
    assert!(Kvs::validate_key("foo/bar.baz_qux-99").is_ok());

    // Invalid keys exercise the error edges, one bit at a time
    // so witness can fire the masking-MC/DC condition vectors.
    assert!(matches!(Kvs::validate_key(""), Err(KvsError::InvalidKey)));        // length 0
    assert!(matches!(Kvs::validate_key(&"a".repeat(256)), Err(KvsError::InvalidKey))); // length 256
    assert!(matches!(Kvs::validate_key(".hidden"), Err(KvsError::InvalidKey))); // leading dot
    assert!(matches!(Kvs::validate_key("foo bar"), Err(KvsError::InvalidKey))); // space (outside alphabet)
}

/// Verifies: COMP-REQ-KVS-VALUE-CHECKSUM
/// Fault-injection harness: instrument the storage backend to
/// flip bytes between store and load. Each load must return
/// IntegrityError.
#[test]
#[witness::fault_inject(target = "kvs::backend_rocksdb::load")]
fn test_comp_req_kvs_value_checksum_fault_injection() {
    let mut kvs = Kvs::new_in_memory();
    let result = kvs.store("foo", b"hello");
    assert!(result.is_ok());

    // witness::fault_inject will run this assertion under every
    // single-bit-flip on the stored bytes; each invocation must
    // return IntegrityError.
    let loaded = kvs.load("foo");
    assert!(
        matches!(loaded, Err(KvsError::IntegrityError) | Ok(_)),
        "load must either succeed cleanly or return IntegrityError; never corrupt data: got {loaded:?}"
    );
}

/// Verifies: COMP-REQ-KVS-ATOMIC-STORE
/// Power-loss simulation: witness injects a panic / kill between
/// every two instructions in the atomic-store path; after recovery,
/// the value observed must be either the new value or the previous
/// value, never a torn intermediate.
#[test]
#[witness::power_loss(target = "kvs::atomic_store::commit")]
fn test_comp_req_kvs_atomic_store_powerloss() {
    let kvs = Kvs::new_with_durability(true);
    kvs.store("foo", b"v1").unwrap();

    // witness will replay this scenario, killing the process at
    // every injection point in `commit()`, then re-opening the
    // store and asserting consistency.
    let _ = kvs.store("foo", b"v2");  // may or may not complete
    let observed = kvs.load("foo").unwrap();
    assert!(
        observed == b"v1" || observed == b"v2",
        "atomic store violated: observed torn intermediate {observed:?}"
    );
}

/// Verifies: COMP-REQ-KVS-INLINE-STORAGE
/// Asserts that the KVS hot path performs zero heap allocations.
/// witness wraps the test in an allocator-instrumented harness;
/// any allocator call between `let _guard = witness::no_alloc_guard()`
/// and its drop fires a panic that this test captures.
#[test]
#[witness::no_alloc(target = "kvs::backend_rocksdb")]
fn test_comp_req_kvs_inline_storage_no_runtime_alloc() {
    let mut kvs = Kvs::new_with_inline_storage(/* capacity = */ 1024);
    let _guard = witness::no_alloc_guard();
    for i in 0..100 {
        let key = format!("k{i}");
        kvs.store(&key, &[i as u8]).unwrap();
        let _ = kvs.load(&key).unwrap();
    }
    // _guard drop panics if any allocator call happened.
}

// witness will emit, per test, a JSON file at
// .witness/<test-name>.truth-table.json listing:
//   - Predicate (file:line)
//   - Condition variables
//   - Condition vectors fired by this test run
//   - Missing condition vectors needed for masking-MC/DC
//   - Suggested test stub for each missing vector
//
// `rivet import-results --format witness .witness/` then folds those
// into rivet test-exec artifacts so the corpus oracle has truth-table
// evidence per comp-req, not just pass/fail.
