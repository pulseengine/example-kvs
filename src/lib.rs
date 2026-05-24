// example-kvs WASM-component implementation of arch/kvs.wit.
//
// In-memory KVS suitable for the worked example — store/load/delete
// against a HashMap, snapshots via copy-on-write. NOT a production
// implementation (no durability, no integrity check yet) — the point
// is that the trait signatures match arch/kvs.wit and the WASM
// component link step proves it.
//
// Eclipse-score's persistency::kvs has a much richer real
// implementation in baselibs_rust + persistency repos; this example
// stays minimal so the focus is on the binary-contract chain.

// wit-bindgen generates these bindings from arch/kvs.wit at build
// time via rules_wasm_component's rust_wasm_component_bindgen rule.
use kvs_component_bindings::exports::pulseengine::kvs::kvs::{Guest, KvsError, SnapshotId};

use std::cell::RefCell;
use std::collections::HashMap;

// Implementation state. WASM components are single-instance per call;
// thread-local interior mutability is fine for the worked example.
thread_local! {
    static STORE: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    static SNAPSHOTS: RefCell<HashMap<SnapshotId, HashMap<String, Vec<u8>>>>
        = RefCell::new(HashMap::new());
    static NEXT_SNAPSHOT_ID: RefCell<SnapshotId> = RefCell::new(1);
}

struct Component;

// The wit-bindgen-generated `Guest` trait names every operation in
// the WIT interface. If we miss one or change a signature, the
// compile fails. That's the binary contract.
impl Guest for Component {
    fn store(key: String, value: Vec<u8>) -> Result<(), KvsError> {
        // COMP-REQ-KVS-KEY-NAMING — see verification/mc_dc_harness.rs
        if !is_valid_key(&key) {
            return Err(KvsError::InvalidKey);
        }
        STORE.with(|s| s.borrow_mut().insert(key, value));
        Ok(())
    }

    fn load(key: String) -> Result<Vec<u8>, KvsError> {
        if !is_valid_key(&key) {
            return Err(KvsError::InvalidKey);
        }
        STORE.with(|s| s.borrow().get(&key).cloned()).ok_or(KvsError::NotFound)
    }

    fn delete(key: String) -> Result<(), KvsError> {
        if !is_valid_key(&key) {
            return Err(KvsError::InvalidKey);
        }
        STORE.with(|s| s.borrow_mut().remove(&key));
        Ok(())
    }

    fn snapshot_create() -> Result<SnapshotId, KvsError> {
        let snapshot = STORE.with(|s| s.borrow().clone());
        let id = NEXT_SNAPSHOT_ID.with(|n| {
            let id = *n.borrow();
            *n.borrow_mut() += 1;
            id
        });
        SNAPSHOTS.with(|s| s.borrow_mut().insert(id, snapshot));
        Ok(id)
    }

    fn snapshot_restore(id: SnapshotId) -> Result<(), KvsError> {
        let snapshot = SNAPSHOTS.with(|s| s.borrow().get(&id).cloned())
            .ok_or(KvsError::SnapshotNotFound)?;
        STORE.with(|s| *s.borrow_mut() = snapshot);
        Ok(())
    }
}

/// COMP-REQ-KVS-KEY-NAMING: keys must be 1..255 chars, alphabet
/// [A-Za-z0-9_./-], not starting with '.'.
fn is_valid_key(key: &str) -> bool {
    if key.is_empty() || key.len() > 255 {
        return false;
    }
    if key.starts_with('.') {
        return false;
    }
    key.bytes().all(|b| {
        b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'/' || b == b'-'
    })
}

// Export the implementation as the WASM component world.
kvs_component_bindings::export!(Component with_types_in kvs_component_bindings);
