+++
id = "smith-kind-text"
part = "smith-items"
kind = "feature"
needs = []
files = ["smith/crates/smith-channel/src/kind.rs", "smith/crates/smith-protocol-channel/src/component.rs", "smith/crates/smith-protocol-channel/src/component/receive.rs", "smith/crates/smith-host-protocol/src/component.rs", "smith/docs/development/debt/D019-channel-component-twins.md", "smith/docs/development/debt/README.md"]
effort = "medium"
minutes = 20
+++

# smith: pay D019, one spelling of a frame kind's name

## Goal

Pay `smith/docs/development/debt/D019-channel-component-twins.md`: the agent's and the host's channel
components name frame kinds through one function in `smith-channel`, with no copy left in either
`component.rs`. No wire change: the refusal's text stays byte for byte.

Acceptance:

- `rg -n 'fn kind_text' smith -g '*.rs'` finds nothing, and `smith_channel::kind::text` is the one
  function.
- `kind::text(0x0100)` reads `kind 0x0100`; `kind::text(0xabcd)` reads `kind 0xabcd`.
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/design/protocol/channel.md`, 3 and 4 (the kinds; nothing changes there).
- `smith/docs/development/debt/D019-channel-component-twins.md`.

## Owns and may change

- Owns: `smith-channel/src/kind.rs`; the two copies and their three call sites
  (`smith-protocol-channel/src/component/receive.rs`, `smith-host-protocol/src/component.rs`).
- A `use` of `kind_text` or of `Box` that the move leaves unused goes.
- `smith-channel` is `no_std` with `alloc`, and already depends on `skein-lib`.

## Steps

1. In `kind.rs`, after `VERSION`:

   ```rust
   /// A kind's name in a refusal's text: `kind 0x` and its four lower-case hex digits.
   #[must_use]
   pub fn text(kind: u16) -> Box<[u8]> { ... }
   ```

   The body is `kind_text`'s, moved as it is, under `use alloc::boxed::Box;`.
2. Both components call `smith_channel::kind::text(kind)`; delete both copies.
3. A `#[cfg(test)] mod tests` at the foot of `kind.rs` with the two values above. Nothing tests this
   text today.
4. `cargo fmt`; `heavy cargo clippy -p smith-channel -p smith-protocol-channel -p smith-host-protocol
   --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-channel -p smith-protocol-channel
   -p smith-host-protocol`; `heavy cargo nextest run -p smith-channel-world`, the world that drives
   both halves.
5. Delete the item and its row; the citations check. Commit and push.

## Report

The common note's items. Break: swap two nibbles in `text` and name the failure.
