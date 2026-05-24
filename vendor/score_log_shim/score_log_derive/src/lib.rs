// SPDX-License-Identifier: Apache-2.0
//
// Proc-macro stand-in: `#[derive(ScoreDebug)]` expands to an empty
// token stream. The companion `score_log` crate carries a blanket
// `impl<T: Debug + ?Sized> ScoreDebug for T`, so every type that
// already (or also-)derives `Debug` automatically satisfies the
// `ScoreDebug` trait bound — no emitted impl required.

use proc_macro::TokenStream;

#[proc_macro_derive(ScoreDebug)]
pub fn derive_score_debug(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
