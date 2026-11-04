+++
id = "file-op-lane"
part = "io"
kind = "feature"
needs = ["window-gone"]
files = ["skein/crates/skein-io/src/file_layer.rs", "skein/crates/skein-io/src/limits.rs", "skein/crates/skein-io/src/tests/file.rs", "skein/testing/skein-world/src/files.rs", "skein/docs/development/debt/D044-io-token-tags.md", "skein/docs/design/io.status.md"]
effort = "high"
minutes = 75
+++

# `Io`'s and `FileIo`'s operation tokens are never equal

## Goal

`Io` names an operation by slot and generation, `FileIo` counts from 1 (`file_layer.rs`, `next_op`), so
their tokens can be equal on one kernel, and a completion can go to the wrong machine. skein-world tags
`FileIo`'s by hand. Make the tokens disjoint where they are minted, and give a service the one test it
needs. (Bug 1.) A change of meaning: `FileIo`'s token values move, so its readers' tests run.

Acceptance:

- `file_layer.rs` has `pub const LANE: u64 = 1 << 59` and `pub const fn owns(op: Token) -> bool`; both mint
  sites (`issue`, `submit_cancel`) go through one private `fn`, minting `LANE | next_op`.
- `limits::flights` is `None` past 2^27 slots, a named constant beside it, so `valid` refuses them and no
  `Io` token has bit 59. (`Id::token` puts the slot in the top 32 bits: `skein-lib/src/id.rs`.)
- skein-world's `files.rs` has no `TAG`: `Driver::owns` is `file_layer::owns`, and submissions and
  completions pass as they are.
- The tests below; the first fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/io.md`, 5 (its opening: "The operation's token says whose"); `skein/docs/design/lib.md`, 4.
- `skein/docs/development/debt/D044-io-token-tags.md`.

## Owns and may change

- Owns: `src/file_layer.rs`, `src/limits.rs`, `src/tests/file.rs`; `skein/testing/skein-world/src/files.rs`,
  its module doc included.
- `skein/docs/design/io.status.md`: remove what remains of this plan's **Not yet** entry for section 5
  (`notes/common.md`).
- `skein/docs/development/debt/D044-io-token-tags.md`: this item stays open, narrowed. Rewrite `What`, `Paid
  when` and `Measure` to what is left: `skein-ui-text/src/terminal.rs`'s `TAG` tells the terminal's `Io` from
  the application's machines, a second `Io` on one kernel, which section 5 leaves to the service; `Measure`
  is 1 at this commit. Do not delete it: `smith/docs/development/debt/D003-host-io-route-table.md` cites it.
- `skein-ui-text/src/terminal.rs` does not change. Neither do the taggers above bit 59, which this
  bit stays below (surveyed): smith's bits 61 to 63 (`smith-local-service/src/service.rs`,
  `credentials/transport.rs`), and jig's `Token::tagged(tag, op.raw())`
  (`jig/testing/jig-test-store-files/src/disk.rs`), which needs a raw token under 2^60.
- `skein_world::files::Driver::owns`' callers (`skein/tests/oauth-accounts/src/sign_in.rs`) keep it.
- A test that pins a limit past the new ceiling as valid (`rg 'sockets' skein/crates/skein-io/src/tests`):
  lower its figure and list it. A test that pins a `FileIo` token's number: take the token from the
  submission, and list it.

## Steps

1. In `src/tests/file.rs`, write `an_io_and_a_file_operation_are_never_the_same_token`: drive one `Io` until
   a flight's slot is reused, so its token is `Token::new(1)` (slot 0, generation 1), beside a `FileIo`'s
   first submission, and `assert_ne!` the two. Run on the old code and quote the failure. Add
   `owns_tells_a_file_operation_from_an_io_one`, and a `limits` row: `flights` at the ceiling is `Some`, one
   socket past is `None`.
2. The lane, the ceiling, then skein-world.
3. `heavy cargo nextest run -p skein-io --lib`, then, each through `heavy cargo nextest run -p`:
   `skein-io`, `skein-world`, `skein-oauth-accounts`, `skein-oauth-accounts-world`, `smith-protocol-machine`,
   `smith-local-shell`, `smith-local-service`, `smith-agent-service`, `jig-protocol-store`,
   `jig-test-store-files`, `jig-client-shell-native`, `temper-box`, `temper-client-shell-cli`,
   `temper-github-app`, and `-p temper --lib`. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`,
   `heavy cargo clippy -p skein-io -p skein-world --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code, and each test whose figure or
pin you changed.
