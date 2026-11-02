// SPDX-License-Identifier: Apache-2.0
//
// Minimal no-op stand-in for eclipse-score/baselibs_rust `score_log`.
// Lets the vendored `rust_kvs` crate at ../../rust_kvs/ compile and
// run its full test suite without pulling in baselibs_rust.
//
// See vendor/score_log_shim/README.md for design notes.

#![forbid(unsafe_code)]

pub mod fmt {
    /// Loggability marker used as a trait bound by `rust_kvs`. The
    /// blanket impl ties it to `Debug`, so every `T: Debug` is also
    /// `ScoreDebug` — and `T: ScoreDebug` implies `T: Debug` via the
    /// super-trait, so the log macros' format_args type-check.
    pub trait ScoreDebug: ::core::fmt::Debug {}
    impl<T: ::core::fmt::Debug + ?Sized> ScoreDebug for T {}
}

// Re-export the derive so `use score_log::ScoreDebug;` resolves to a
// proc-macro (matching upstream's two-namespace pattern).
pub use score_log_derive::ScoreDebug;

// Log macros: evaluate format_args (keeps locals "used", type-checks
// the arguments against Debug/Display) but discard the result.
// A production safety build replaces this with a vetted logger.

#[macro_export]
macro_rules! fatal {
    (context: $ctx:expr, $($arg:tt)+) => {{
        let _ = $ctx;
        let _ = ::core::format_args!($($arg)+);
    }};
}

#[macro_export]
macro_rules! error {
    (context: $ctx:expr, $($arg:tt)+) => {{
        let _ = $ctx;
        let _ = ::core::format_args!($($arg)+);
    }};
}

#[macro_export]
macro_rules! warn {
    (context: $ctx:expr, $($arg:tt)+) => {{
        let _ = $ctx;
        let _ = ::core::format_args!($($arg)+);
    }};
}

#[macro_export]
macro_rules! info {
    (context: $ctx:expr, $($arg:tt)+) => {{
        let _ = $ctx;
        let _ = ::core::format_args!($($arg)+);
    }};
}

#[macro_export]
macro_rules! debug {
    (context: $ctx:expr, $($arg:tt)+) => {{
        let _ = $ctx;
        let _ = ::core::format_args!($($arg)+);
    }};
}

#[macro_export]
macro_rules! trace {
    (context: $ctx:expr, $($arg:tt)+) => {{
        let _ = $ctx;
        let _ = ::core::format_args!($($arg)+);
    }};
}
