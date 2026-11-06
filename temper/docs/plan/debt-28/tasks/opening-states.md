+++
id = "opening-states"
part = "file-layer"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/file_layer/private.rs"]
effort = "high"
minutes = 60
+++

# `Opening` is one enum, each state holding what it holds

## Goal

Part of skein's D049. `Opening` keeps its 9 phases beside the `Option`s `parent`, `file` and `terminal`, and the flag `made`. It's one lifecycle, so it's one enum (`skein/docs/design/io.md`, 5; `skein/docs/foundation/programming-model.md`, 5.4). Private to the file layer, so no reader sees it.

Acceptance:

- `Opening`'s `phase`, those 3 `Option`s and `made` are gone: the enum's variants hold them, and `Opening` keeps `owner` beside its one state.
- `refuse_file` starts at the state that closes the file, with no placeholder: the `Action::Make { mode: 0 }`, empty name and `Phase::Stating` it builds today have gone.
- A match over the new enums lists every variant, with no `_`, no `matches!` and no `if let`, in `private.rs`.
- No test changes, in any package.
- `private.rs` is at most 520 lines (423 now), counted without inline `#[cfg(test)]` items. This task is the request's "D049's states apart": it may grow within that.

The item stays open: `file-thin-gone` deletes it.

## Inputs

- `skein/crates/skein-io/src/file_layer/private.rs`, whole.
- `skein/docs/design/io.md`, 5 and 5.3.
- `skein/docs/foundation/programming-model.md`, 5.4: the idiom.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns `private.rs`. In `skein-io/src/file_layer.rs`, only a line that reads an `Opening` field or phase and must change for it. `store.rs` is `store-states`'s.

## Steps

A sketch, whose variants' meaning binds and whose names are yours:

```rust
enum State {
    ParentOpening { name, action: Start },              // Make { mode } | Private { user }
    Making { parent, name, then: Then },                // Then: Made | Open { user }
    SyncingParent { parent, name, then: Then },
    Opening { parent, name, user, may_make: MayMake },  // Try | Tried, which replaces `made`
    Stating { parent, file, user },
    DeniedOpening { parent: Held, name, denied: Event, check: Check },  // Check: Root { user } | File { user }
    DeniedStating { parent: Held, file, denied: Event, check: Check },
    ClosingFile { parent: Held, terminal: Event },      // where `refuse_file` starts
    ClosingParent { parent, answer: Answer },           // Answer: Terminal(Event) | Admit { file, user }
}
enum Held { Parent(Fd), Gone }  // a denied load and `refuse_file` have no parent
```

**What the rewrite must keep,** since no test will catch all of it: a cancel during the closing of an admitted root's parent can still abandon (today `terminal.is_none()`), and `stopped` then closes the file. The order of submissions and the terminal event doesn't change.

Each cell is a small handler taking the source state's data by value, as 5.4 shows.

No new test: the types hold the rule, and `skein-io/src/tests/private.rs` and `skein/tests/io`'s private world hold the behaviour.

`heavy cargo nextest run -p skein-io`; the package for `skein/tests/io` in `default`, then `--profile fuzzy -E 'test(private)'`, if the count isn't 0. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`. Commit, push.

## Report

As your text says, with the line count.
