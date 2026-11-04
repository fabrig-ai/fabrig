+++
id = "held-refusal"
part = "listeners"
kind = "feature"
needs = ["listening-check"]
files = [
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/tests/box_kinds.rs",
]
effort = "medium"
minutes = 25
+++

# A descriptor the box holds is not `LISTEN_FDS`'s

## Goal

`listeners_refusal` (`temper/crates/temper/src/lib.rs`) says `LISTEN_FDS` for every descriptor,
so the box's check of its own held listeners misled the first macOS red. Word a refusal by where the
descriptor came from, and give the box's check a function of its own that a test reaches.

Acceptance:

- A refusal of a descriptor the box holds reads `held listener: ...`, never `LISTEN_FDS`, and a test
  through the box's check shows it.
- Every refusal of an inherited descriptor reads as it did, `LISTEN_FDS: ...`: the startup tests that
  read those words pass unchanged.
- The words are the table in `notes/contracts.md`, and a test holds the two failed-call rows.
- The new tests bite, and `temper`'s checks and Darwin's clippy pass (`notes/common.md`).

## Inputs

- `skein/docs/design/shell.md`, 6.6: `listener_at`'s last sentence.
- `temper/docs/design/protocol/setup.md`, 7.1: the box holds its children's listeners.
- `temper/docs/plan/macos-box-start/notes/common.md`
- `temper/docs/plan/macos-box-start/notes/contracts.md`: the words, as binding.

## Owns and may change

- Owns: `listeners_refusal` and its callers in `temper/crates/temper/src/lib.rs`, and the held check
  in `run_configuration` (`temper/crates/temper/src/box_command.rs`).
- May change: `temper/crates/temper/src/tests/box_kinds.rs`, for the tests. If that module reads
  wrongly for them, a new `tests/` module beside it with its `mod` line in `tests.rs` is fine.
- `fake.rs`, `forge.rs` and `serve` keep calling `listeners_refusal`: those descriptors are inherited.

## Steps

In `lib.rs`:

```rust
fn listener_words(why: skein_shell::ListenersRefusal) -> String;   // every arm, with no prefix
fn listeners_refusal(why: skein_shell::ListenersRefusal) -> String; // "LISTEN_FDS: {words}"
fn held_refusal(why: skein_shell::ListenersRefusal) -> String;      // "held listener: {words}"
```

In `box_command.rs`, take the check out of `run_configuration`:

```rust
/// Checks the listeners the box holds itself, which it was never given.
fn held_listeners(held: Vec<(Fd, NonZeroU16)>) -> Result<Vec<(Fd, SocketAddr)>, String>;
```

Tests:

- `a_held_descriptor_is_refused_as_held`: a descriptor that is no socket (`/dev/null`, opened in the
  test) through `held_listeners` gives `held listener: fd N is not a socket`.
- `a_refusal_names_whose_descriptor_it_was`: `held_refusal` and `listeners_refusal` over
  `Why::Check` with `ListenerCheck::Listening` and over `Why::Kernel`, each compared whole, as the
  table has them.

Then the checks your implementer text names, Darwin's clippy, commit and push.

## Report

As your implementer text says.
