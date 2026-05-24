// Empty stub so cargo doesn't try to compile src/lib.rs (which depends
// on Bazel-generated `kvs_component_bindings` and cannot be built by
// plain cargo). The cargo-shim crate only exists for crate_universe's
// external-dep resolution; see /Cargo.toml header.
