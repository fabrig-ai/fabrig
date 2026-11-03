# wasm32-jig: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `jig/AGENTS.md`. The plan changes lint attributes in jig's step crates
  and nothing else: no type, signature, variant, body, manifest, `clippy.toml` or document.
- **A lint exception is a scoped `expect`, never `allow`** (`AGENTS.md`, 2). Where an
  expectation holds on one target and not the other, the `expect` sits inside a `cfg_attr` on
  the target where the lint fires: `#[cfg_attr(not(target_arch = "wasm32"), expect(<lint>,
  reason = "..."))]` for a lint that fires on the host only (the form skein-codec's
  `cfg_attr(not(test), expect(dead_code, ...))` already uses). Its lint and reason stay as they
  were.
- **wasm32 clippy is lib-only:** `cargo clippy --target wasm32-unknown-unknown -p
  temper-playground` lints every workspace crate it builds (clippy wraps every workspace
  member), without tests. A file under `src/tests/` or a `#[cfg(test)]` module never reaches it.
- **The wasm32 target** (`wasm32-unknown-unknown`): if `heavy` reports it missing (`can't find
  crate for \`core\``, "the target may not be installed"), stop: report `Failed: wasm32 target
  not installed`, with the host from heavy's output. Never `rustup target add`: a builder's
  toolchain is the user's.
- **No tests run.** This departs from the implementer's text (the focused tests of the packages
  changed and their dependents): every edit keeps the host's build as it was, since each
  `cfg_attr` keeps the same expectation on the host, so no test can tell. The native clippy of
  the changed packages is the host's check.
- **Another tree's code is not this plan's:** a warning or error located under `skein/`,
  `smith/` or `temper/` is listed in the report's `## Left` (tree, package, file and line, lint),
  never fixed. Plan playground-browser owns `temper-client-shell-web` and `temper-playground`,
  and the list `.config/gate/wasm32.txt`.
- Read heavy's logs by `grep`, never whole. One commit per coherent step, each saying why.
  Push your branch.
