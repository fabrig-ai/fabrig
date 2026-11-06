+++
id = "store-states"
part = "file-layer"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/file_layer/store.rs"]
effort = "high"
minutes = 120
+++

# `Store` is one enum, each state holding what it holds

## Goal

Part of skein's D049. `Store` keeps its 20 phases beside 8 `Option`s (`parent`, `old`, `temporary`, `check`, `temp_name`, `residue`, `bytes`, `failure`) and the flags `temp_exists` and `committed`. The cut read them: it's one lifecycle, since `old`, the temporary and `check` are never open together. So it's one enum (`skein/docs/design/io.md`, 5; `skein/docs/foundation/programming-model.md`, 5.4). Private to the file layer, so no reader sees it.

Acceptance:

- The phase enum, those 8 `Option`s and the 2 flags are gone from `Store`: the enum's variants hold them. `Store` keeps what holds for the whole request (`owner`, `target`, `expected`, `produced`, `no_follow`, `private_owner`) as fields beside its one state. `mode` and the rest of your sketch's fields follow where the code needs them, and your report names any left beside the state, with why.
- A match over the new enums lists every variant, with no `_`, no `matches!` and no `if let`, in `store.rs`.
- No `.expect` or `unreachable!` is left that asserts a field a state now holds.
- No test changes, in any package. All of `skein-io`'s store tests and `skein/tests/io`'s worlds drive the machine through requests and completions, and none names a phase.
- `store.rs` is at most 1,100 lines (919 now), counted without inline `#[cfg(test)]` items. This task is the request's "D049's states apart": it may grow within that.

The item stays open: `file-thin-gone` deletes it.

## Inputs

- `skein/crates/skein-io/src/file_layer/store.rs`, whole.
- `skein/docs/design/io.md`, 5.
- `skein/docs/foundation/programming-model.md`, 5.4: the idiom.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns `store.rs`. In `skein-io/src/file_layer.rs`, only a line that reads a `Store` field or phase and must change for it: `Store` sits inline in `Work`. Anything else there is another task's. `file_layer/private.rs` is `opening-states`'s.

`Pending::LoadOpening { loading: Option }` and `Loading.reopened` in `file_layer.rs` have the same shape, and the item doesn't name them: leave them, and list them under `## Left`.

## Steps

A sketch, whose variants' meaning binds and whose names are yours:

```rust
enum State {
    ParentOpening { bytes },
    OldOpening { parent, bytes }, OldStating { parent, old, bytes }, OldClosing { parent, old, bytes },
    TempCreating { parent, bytes, name, attempts: u8 },
    TempWriting { parent, temp, name },   // bytes now live in the write under way
    TempSyncing { parent, temp, name }, TempClosing { parent, temp, name },
    CheckOpening { parent, name }, CheckStating { parent, check, name },
    CheckReading { parent, check, name, seen: Sha256, seen_bytes: u64 },
    CheckClosing { parent, check, name, verdict: Verdict },  // Matches | Refused(Failure)
    Renaming { parent, name },
    DirectorySyncing { parent }, ParentClosing { parent },   // committed
    Settling { failure: Failure, step: Settle },
}
enum Settle {  // in today's cleanup order; no parent left is terminal at once
    ClosingCheck { parent, check, name }, ClosingOld { parent, old },
    ClosingTemp { parent, temp, name }, Removing { parent, name },
    ClosingParent { parent, left: Left },
}
enum Left { Nothing, Committed, Residue(Residue) }  // exclusive: a commit clears temp_exists
```

**What the rewrite must keep,** since no test will catch all of it:

- A later cleanup error overwrites the failure the store already holds.
- `Store::deadline` is `None` once a failure is set, including while the check closes with a verdict of refusal.
- The order of cleanup, step for step, and so the order of the submissions and the terminal event.
- Debug output hides contents: `skein-io/src/tests/store.rs`'s canaries pin that.

Each cell is a small handler taking the source state's data by value, with the terminal state left in place by `mem::replace`, as 5.4 shows. A handler that needs an impossible cell writes `unreachable!("why")`, for cells the loop's own rules make unreachable only.

No new test: the types hold the rule, and the machine's tests hold its behaviour. **A change of meaning runs its readers' tests,** should you find one: the order of events is what to watch, and it doesn't change.

`heavy cargo nextest run -p skein-io`; `heavy cargo nextest run -p skein-shell`, whose startup store is a reader; the package for `skein/tests/io` in `default`, then `--profile fuzzy -E 'test(files)'`, checking the count isn't 0; `skein/tests/io/tests/suite/files.rs`'s peak ceilings must hold, and a ceiling that fails stops and asks. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`. Commit, push.

## Report

As your text says, with the line count, the fields left beside the state and why, and anything under `## Left`.
