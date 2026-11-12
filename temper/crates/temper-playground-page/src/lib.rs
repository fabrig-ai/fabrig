//! The playground's entry and frame (temper/docs/design/playground.md, 3 and 6).
#![forbid(unsafe_code)]

#[cfg(any(target_arch = "wasm32", test))]
mod playground;
