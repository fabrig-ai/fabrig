+++
id = "settings-name"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/tests/settings/application_proposals.rs", "jig/crates/jig-domain-tasks/src/closing.rs"]
effort = "medium"
minutes = 30
+++

# jig: the value-mismatch abort comes back, `named` answers a `Name`, and closing gates before it clones

## Goal

Finding 8 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, checked against the code. Three small things:

1. `jig-domain-core/src/settings/application.rs`'s `translate` answers `None` for a `Setting::Application` whose value isn't `Value::Application`, so `keep_proposal` files it under jig where it used to abort. The arm aborts again: `unreachable!("checked application value")`. `check` refuses such an item at the entrance, so this guards that promise. `translate` still answers `None` for a setting that isn't an application's, through `named`: `keep_proposal` reads that as a jig item.
2. `named` returns `(u32, u32, u32)`, read in one order by `check`, `start`, `drop_project`, `recorded`'s callers and `application_failure`, and by `restore` under another name: its `let (number, scope, _) = name` calls the kind `number`. A small private `struct Name { kind, scope, number }`, `Clone, Copy`, replaces the tuple everywhere in this file, including `missing`'s pairs and `application_failure`'s and `recorded`'s parameter. `restore`'s `number` becomes `kind`.
3. `jig-domain-tasks/src/closing.rs`'s `progress` clones `closing.ending` before `begin_effects` can answer `false`. Gate first: the four-part test moves to a small function, `progress` asks it, and clones only past it. `begin_effects` is left with the transition and answers nothing. `changed |=` stays true exactly when the transition ran.

Acceptance:

- A `#[should_panic(expected = "checked application value")]` test in `src/tests/settings/application_proposals.rs` calls `s::application::keep_proposal` on a `Proposal` with one `Setting::Application` item whose edit sets a `Value::Charter`. It goes red if the arm answers `None`: break it that way once and report it.
- No `(u32, u32, u32)` is left in `application.rs`.
- `jig/docs/design/domain/core.status.md` stays as it is: it reads "No gap" and records no such departure, and no other status of `jig/docs/design/domain/` does.
- `drop_project` and `restore` keep skipping a setting `named` answers `None` for: that's no value mismatch, and the request doesn't bring those aborts back.
- `heavy cargo nextest run -p jig-domain-core --lib`, `-p jig-domain-tasks --lib`, `-p jig-core-world` and `-p jig-tasks-world` pass with no assertion changed.
- Net production lines don't rise by more than about 10, tests about 15.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, finding 8
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`, "What": a guard of another step's promise stays. The item doesn't close and doesn't change.
- `jig/docs/design/domain/engine.md`, 17.5, 17.6

## Owns and may change

- **Owns:** the files in the header. `begin_effects` has one caller.
- **Other drafts:** `note-parts` and `ui-chat-draft` change other files of `jig-domain-core`, none of these.
- **If `keep_proposal` can't be called from that test** (a private path, a `Proposal` it can't build), go through `propose` with such an item only if `check` lets it pass; if neither reaches the arm, drop the test, say so, and report the search that shows `check` refuses it first.

## Steps

1. The test; see it fail on the old `None`. The arm; the test passes.
2. `Name`, through the file.
3. The gate in `closing.rs`.
4. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core -p jig-domain-tasks --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; what you broke and what failed.
