# `score_log_shim/`

A minimal, dependency-free stand-in for the `score_log` and
`score_log_derive` crates from `eclipse-score/baselibs_rust`.

The vendored `rust_kvs` crate at `../rust_kvs/` uses:

- Six log macros (`fatal!`, `error!`, `warn!`, `info!`, `debug!`,
  `trace!`) that accept a `context: $expr,` argument followed by
  `format_args!`-style trailing tokens.
- A `score_log::fmt::ScoreDebug` trait used as a trait bound in
  generic code.
- A `#[derive(score_log::ScoreDebug)]` derive that participates in
  `Debug`-shape printing.

This shim is the smallest set of definitions that lets `rust_kvs`
compile and run its full test suite **without** also pulling in
`baselibs_rust` (and its `stdout_logger`, in-house signal handlers,
etc.). It is intentionally not a port of `score_log`: the logging
macros are no-ops, and `ScoreDebug` is wired to the standard library's
`Debug` trait.

Real safety builds would replace this with a vetted logger; the
shim is only here to keep this *example* dependency-light.

## Layout

```
score_log_shim/
├── score_log/         # rlib crate with macros + ScoreDebug trait
│   ├── Cargo.toml
│   └── src/lib.rs
└── score_log_derive/  # proc-macro crate for #[derive(ScoreDebug)]
    ├── Cargo.toml
    └── src/lib.rs
```

The proc-macro crate has zero non-std dependencies (no `syn`,
no `quote`) — it does small string surgery on the token stream
to extract the type name and emits a `Debug` impl that prints
`{type_name} {{ .. }}`.
