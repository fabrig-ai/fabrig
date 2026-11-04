# wasm32-lints: temper-playground and smith-agent-shell pass wasm32 clippy

The gate's wasm32 list (`.config/gate/wasm32.txt`) has every package the designs build for wasm32 except two: `temper-playground` and `smith-agent-shell`. `temper/docs/design/playground.status.md`, 9, and `smith/docs/design/shell.status.md`, 2.5, name what fails. All of it is lint-level, in jig's and smith's crates:

- `unfulfilled_lint_expectations` for `clippy::trivially_copy_pass_by_ref` on `Index::publish` and `Index::follow` (`jig/crates/jig-domain-core/src/public/index.rs:115` and `130`);
- an unmet `clippy::large_enum_variant` expectation on `Source` (`smith/crates/smith-local-service/src/credentials.rs:27`);
- in `smith/crates/smith-agent-shell/src/config/`, what only `cfg(unix)` code reads is unused on wasm32: the import `skein_tls as tls` (`native.rs:10`), the parameter `errors` (`native.rs:31`), the constant `CERTIFICATES` (`native.rs:74`), `TrustDocument`'s fields (`document.rs:185`) and those of `EndpointTransport::Tls` (`endpoints.rs:39`).

## The task

Make both pass:

- `cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`
- `cargo clippy --target wasm32-unknown-unknown -p smith-agent-shell -- -D warnings`

For a lint expectation met on one target only, prefer `cfg_attr(not(target_arch = "wasm32"), expect(..., reason = "..."))`, or make the code right for both. Put unix-only items behind the same `cfg` as what reads them. Never `#[allow]` (`AGENTS.md` 2). Native clippy stays green.

Then add both packages to `.config/gate/wasm32.txt`, and remove the two status entries' wasm32 items (`playground.status.md` 9, `shell.status.md` 2.5).

One task; no design. jig is shared with hearth: change only the lint attributes there, no behaviour.
