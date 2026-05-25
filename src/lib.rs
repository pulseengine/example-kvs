// example-kvs WASM-component impl of arch/kvs.wit, wired to the
// vendored eclipse-score rust_kvs.
//
// The WIT contract takes Vec<u8> values; rust_kvs's KvsValue has no
// Bytes variant, so values are encoded as KvsValue::Array(Vec<U32>)
// in the underlying store. Lossless round-trip; not space-efficient,
// but the point here is to prove the upstream code is the actual
// implementation that links into the WASM component (not a toy
// stub), not to optimize the encoding.
//
// Backend: a thin InMemoryBackend (KvsBackend impl) lives at the
// bottom of this file. JsonBackend depends on std::fs::rename and
// would need WASI filesystem permissions to function inside a
// component; for a self-contained example, in-memory is enough to
// demonstrate the code-path.

use kvs_component_bindings::exports::pulseengine::kvs::kvs::{Guest, KvsError, SnapshotId};

use rust_kvs::error_code::ErrorCode;
use rust_kvs::kvs_api::{InstanceId, KvsApi, KvsDefaults, KvsLoad};
use rust_kvs::kvs_backend::KvsBackend;
use rust_kvs::kvs_builder::KvsBuilder;
use rust_kvs::kvs_value::{KvsMap, KvsValue};

use std::cell::RefCell;
use std::sync::Mutex;

struct Component;

impl Guest for Component {
    fn store(key: String, value: Vec<u8>) -> Result<(), KvsError> {
        with_kvs(|kvs| {
            kvs.set_value(key, bytes_to_kvs_value(&value))
                .map_err(map_err)
        })
    }

    fn load(key: String) -> Result<Vec<u8>, KvsError> {
        with_kvs(|kvs| {
            let v = kvs.get_value(&key).map_err(map_err)?;
            kvs_value_to_bytes(&v).ok_or(KvsError::NotFound)
        })
    }

    fn delete(key: String) -> Result<(), KvsError> {
        with_kvs(|kvs| kvs.remove_key(&key).map_err(map_err))
    }

    fn snapshot_create() -> Result<SnapshotId, KvsError> {
        // The in-memory backend does not implement durable snapshots;
        // return a static id 0 to satisfy the WIT signature. A real
        // deployment swaps in a backend that supports snapshot_count.
        Ok(0)
    }

    fn snapshot_restore(_id: SnapshotId) -> Result<(), KvsError> {
        Err(KvsError::SnapshotNotFound)
    }
}

// ── adapters between WIT and rust_kvs ────────────────────────────────

fn map_err(e: ErrorCode) -> KvsError {
    match e {
        ErrorCode::KeyNotFound | ErrorCode::FileNotFound => KvsError::NotFound,
        ErrorCode::InvalidSnapshotId => KvsError::SnapshotNotFound,
        _ => KvsError::InvalidKey,
    }
}

fn bytes_to_kvs_value(bytes: &[u8]) -> KvsValue {
    // Lossless: each byte becomes a U32 entry in an Array.
    KvsValue::Array(bytes.iter().map(|b| KvsValue::U32(u32::from(*b))).collect())
}

fn kvs_value_to_bytes(v: &KvsValue) -> Option<Vec<u8>> {
    if let KvsValue::Array(items) = v {
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            if let KvsValue::U32(n) = item {
                if *n > u32::from(u8::MAX) {
                    return None;
                }
                out.push(*n as u8);
            } else {
                return None;
            }
        }
        Some(out)
    } else {
        None
    }
}

// ── thread-local Kvs (single instance for the WASM module) ───────────

thread_local! {
    static KVS_CELL: RefCell<Option<rust_kvs::kvs::Kvs>> = const { RefCell::new(None) };
}

fn with_kvs<R>(f: impl FnOnce(&rust_kvs::kvs::Kvs) -> R) -> R {
    KVS_CELL.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            let kvs = KvsBuilder::new(InstanceId(0))
                .defaults(KvsDefaults::Ignored)
                .kvs_load(KvsLoad::Ignored)
                .backend(Box::new(InMemoryBackend::default()))
                .build()
                .expect("KvsBuilder::build for in-memory backend");
            *borrow = Some(kvs);
        }
        f(borrow.as_ref().unwrap())
    })
}

// ── minimal in-memory backend (KvsBackend impl) ──────────────────────
// Replaces JsonBackend so we don't need WASI filesystem permissions.
// All snapshot operations are no-ops; load returns empty (fresh state).

// Single-instance store — this WASM component only ever opens
// InstanceId(0), so we don't need a map of maps; just one KvsMap
// behind a Mutex.
#[derive(Debug, Default)]
struct InMemoryBackend {
    store: Mutex<Option<KvsMap>>,
}

// KvsBackend's PartialEq super-trait is only used to detect
// parameter mismatches inside KvsBuilder::compare_parameters; for
// a single-instance WASM module, all InMemoryBackend instances are
// equivalent. (Mutex itself doesn't implement PartialEq.)
impl PartialEq for InMemoryBackend {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl KvsBackend for InMemoryBackend {
    fn load_kvs(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: rust_kvs::kvs_api::SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        let m = self.store.lock().map_err(|_| ErrorCode::MutexLockFailed)?;
        Ok(m.clone().unwrap_or_default())
    }

    fn load_defaults(&self, _instance_id: InstanceId) -> Result<KvsMap, ErrorCode> {
        Ok(KvsMap::new())
    }

    fn flush(&self, _instance_id: InstanceId, kvs_map: &KvsMap) -> Result<(), ErrorCode> {
        let mut m = self.store.lock().map_err(|_| ErrorCode::MutexLockFailed)?;
        *m = Some(kvs_map.clone());
        Ok(())
    }

    fn snapshot_count(&self, _instance_id: InstanceId) -> usize {
        0
    }

    fn snapshot_max_count(&self) -> usize {
        0
    }

    fn snapshot_restore(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: rust_kvs::kvs_api::SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        Err(ErrorCode::InvalidSnapshotId)
    }
}

// Export the implementation as the WASM component world.
kvs_component_bindings::export!(Component with_types_in kvs_component_bindings);
