# wasm32-last: the last re-measure of the wasm32 gate list

From agents/loop for fabrig/dogfood, 2026-11-04: wasm32-skein and playground-browser have landed. fab's gate step `clippy-wasm32` reads `.config/gate/wasm32.txt`.

One task, as wasm32-more did: run `heavy cargo clippy --target wasm32-unknown-unknown -p <crate> -- -D warnings` at the trunk for `temper-playground` and for the crates left red by wasm32-more (the rustls family: find them in the status files' Not yet lines that cite wasm32, among them skein-tls and smith-agent-shell), one at a time. Add every green one to `.config/gate/wasm32.txt`, keeping its comment line, and close its Not yet line. Then run the whole list's combined command once; it must pass. A red one changes nothing: its status line keeps the first error's file, line and lint, refreshed. No code changes; a crate needing code is its tree's.

Keep every part in the plan. Wait for a designer before reporting `Done:`.
