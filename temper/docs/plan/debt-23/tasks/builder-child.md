+++
id = "builder-child"
part = "builder-lifecycles"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-protocol-builder/src/held.rs", "hearth/crates/hearth-protocol-builder/src/git.rs", "hearth/crates/hearth-protocol-builder/src/events.rs", "hearth/crates/hearth-protocol-builder/src/lib.rs", "hearth/crates/hearth-protocol-builder/src/read.rs"]
effort = "high"
minutes = 60
+++

# hearth-protocol-builder: `git::Child`'s lifecycle is an enum, and `Held` says each resource's

## Goal

First of hearth D006, in its order. `git::Child` holds `process`, `output`, `error` and `input` as `Option`s, `closing` and `closed` as flags, and a `Closing` struct of four more. Each resource takes a `Held`, the flags and the struct go, and `Resolve` will read the same type after you. About -15 net lines.

The sketch binds; `Child` is private to the crate:

```rust
enum Stage { Live, Closing }   // replaces closing; once Closing, it stays
struct Child { owner, command, tail, fault, exit, output_state, error_state,
    failed: bool,              // the answer's verdict, read at finish
    stage: Stage, process: Held, output: Held, error: Held, input: Held }
```

`process.is_none() && closed` is `process == Held::Closed`. `done()` is process `Closed` with no pipe holding a token. `roles` is read only in `spawned`: if it derives from `command`, it goes, with its allocation and a `_` arm.

Acceptance:

- `rg 'closing: bool|closed: bool|closing_resources|struct Closing' hearth/crates/hearth-protocol-builder/src/git.rs` reads nothing.
- The tests in the note pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header: `lib.rs` in its lines that touch `Child`'s fields, `events.rs` in its 4, `read.rs` in its one `child.input`.
- **`failed` stays a field.** `refused()` can arrive while closing, and `read.failed()` is folded in on every pass, so it can't sit inside a variant as a fixed value.
- **`stream_up` checks in order:** closing first, then fault record, stderr, drain-or-failed, then normal. That stays.

## Steps

1. `Held`, with `mod held;` in `lib.rs`. A small table test in `src/tests` only if `request_close` has a rule the worlds don't reach; then it bites, as the common note says.
2. `Child`, then its readers.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; the tests. Commit and push.

## Report

The common note's items; whether `roles` went.
