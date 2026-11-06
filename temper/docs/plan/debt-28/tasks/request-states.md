+++
id = "request-states"
part = "request-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-lib/src/requests.rs", "skein/docs/design/lib.status.md", "skein/docs/development/debt/D025-request-table-in-lib.md"]
effort = "high"
minutes = 70
+++

# The request table stays in lib, its entry in lifecycle states

## Goal

Pay skein's D025. The cut decided the home: `RequestTable` stays in skein-lib, and `skein/docs/design/lib.md`, 12, now says why. Moving it would add a crate and remove no code, and jig's step crates may hold nothing generic. So this task does the other half: `Entry`'s flags and its `progress` option become lifecycle states (`skein/docs/foundation/programming-model.md`, 5.4), which section 12 now names.

Acceptance:

- `Entry` has no `restored`, `save` or `changed` field, and its `progress` is no `Option`. A combination that can't happen today (`restored` with `save`, `changed` with no progress) can't be written.
- `.expect("a change has a progress value")` has gone, since the state holds the value.
- `requests.rs` has no `if let` or `matches!` over the new enums, and their matches list every variant.
- The public API of `requests.rs` doesn't change, and neither does any test, by this search: `git diff --stat` shows no test file, except a memory pin that shrinks.
- `skein/docs/development/debt/D025-request-table-in-lib.md` is deleted, and the entry citing it leaves `skein/docs/design/lib.status.md`, with its section number's other entries as they are.

Aim for fewer lines. The cap: `requests.rs` grows by at most 20 lines, its inline `#[cfg(test)]` items, if any, counted apart. Your report gives the net.

## Inputs

- `skein/docs/development/debt/D025-request-table-in-lib.md`, whole.
- `skein/docs/design/lib.md`, 12.
- `skein/docs/foundation/programming-model.md`, 5.4.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header. Nothing outside `requests.rs` reads `Entry` or `State`: they're private.

- **A memory pin.** `skein/tests/lib/tests/suite/memory.rs`'s `a_request_table_grows_within_its_held_and_peak_bounds` should hold or shrink, since two bools and an `Option` with a bool beside it become two small enums. If a pin shrinks, update it and say so. If one grows, stop and ask.
- **A digest.** `RequestTable`'s `Hash` feeds a digest. `skein-lib/src/tests/digest.rs` asks only that it changes with each request, which holds. A digest pinned to a value elsewhere is a question. This is a change of meaning for that digest: run the tests of `jig-client-domain`, the table's reader.
- **`anchor`** stays an `Option`, as `lib.md`, 12, says: a value learned once and read beside all three lifecycles, not a phase. Renaming it would pay nothing. `backoff`, and the table's own `up`, `scope` and `retention`, aren't named by the item: they stay.

## Steps

The inside is yours; the enums' meaning binds. A sketch, with the names free:

```rust
/// Whether the asker's store holds this request yet.
enum Keeping {
    /// Asked; its record has not been taken. Takes a save.
    Unsaved,
    /// Asked; its record has been taken.
    Saved,
    /// Restored: needs known retention before it may leave, and has nothing to save until its first sending.
    Restored,
}

/// What the owner has been told.
enum Telling {
    /// Nothing to say yet.
    Silent,
    /// A change not yet taken.
    Changed(RequestProgress),
    /// The last one taken.
    Told(RequestProgress),
}
```

Where the code reads today:

- `restored` at admission and in eligibility: `Keeping::Restored`.
- `save: !restored`: the initial `Unsaved` or `Restored`.
- `entry.save = false` on a final answer, on `not_kept`, in `unknown` and when a save is taken: a transition to `Saved`, which leaves `Restored` as it is, since a restored entry never had `save` set and stays restored for eligibility. Check that with every site: a restored entry still emits its first-send save through `first_sent.is_none()`, as it does now.
- `entry.changed = false` on a final answer and on `not_kept`: `Changed(p)` becomes `Told(p)`, and `Silent` stays. That keeps today's value of `progress`.
- `Entry::progress(p)`: a change only if the last value isn't `p`, whether it was taken or not, as `self.progress != Some(progress)` says today.
- `insert`'s `progress: if eligible { None } else { Some(Parked) }, changed: !eligible`: `Silent`, or `Changed(Parked)`.
- `progress()`'s take: `Changed(p)` hands back `p` and becomes `Told(p)`.

Follow the idiom in 5.4 where a small handler reads better than an assignment. The table in the module's header comment doesn't change, since behaviour doesn't. No new test: the world in `skein/tests/lib/src/request_table.rs` and `skein-lib/src/tests/requests.rs` hold the behaviour, and the types hold the rule.

`heavy cargo nextest run -p skein-lib --lib -E 'test(requests)'`; `heavy cargo nextest run -p skein-lib-tests -E 'test(request)'`, the world in `skein/tests/lib`, with the memory pin you read, checking the count isn't 0; the same with `--profile fuzzy`; `heavy cargo nextest run -p jig-client-domain`. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-lib --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the net lines and whether a pin moved.
