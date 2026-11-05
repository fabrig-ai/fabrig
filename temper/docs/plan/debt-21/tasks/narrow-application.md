+++
id = "narrow-application"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/routing/settings/restore.rs"]
effort = "high"
minutes = 75
+++

# jig-domain-core: an application item's pairing is matched once

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. An application setting's `Setting` and `Edit` are matched, and then matched again when they come back. A total translation and a key carrying the name remove thirteen sites.

Acceptance:

- `rg -c 'unreachable!'` reads 3 in `jig/crates/jig-domain-core/src/settings/application.rs` (`Starting`'s) and 1 in `routing/settings/restore.rs`.
- `heavy cargo nextest run -p jig-domain-core --lib -E 'test(settings)'` and `heavy cargo nextest run -p jig-core-world --test suite` pass with no assertion changed. The suite's `application_settings`, `settings_restore` and `restore_pages` pin this. **Refusal order outranks shape faults:** two loops stay, for that reason.
- `heavy cargo check -p jig-ops-domain -p temper-engine-domain --tests` passes.
- Net production lines don't rise by more than the cut's estimate of 10.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/domain/engine.md`, 17.5 and 17.10; `jig/docs/design/domain/connectors.md`, 17

## The sites

- `settings/application.rs:216, 223, 415, 614, 777, 804, 1004, 1071`:
  - `translate(item) -> Option<Item>` replaces the outer `match item.setting` lists.
  - A total `named(setting) -> Option<(kind, scope, number)>`.
- `routing/settings/restore.rs:306, 332, 431, 479`:
  - `RestoreEdit::Application` carries the name. It's `pub(crate)`.
  - A first pass turns `Judged` into a private `Answered { Same, Accept }`.
- `routing/settings/restore.rs:361` becomes `kind_of(..).expect("checked kind")`: the one exception to the note's rule, keeping its abort, since a checked edit's kind is a lookup, not a re-match.

## Owns and may change

- **Owns:** the two files.
- **`Setting`, `Value` and `Item`'s public shapes stay.**
- **Stay off** every other file in the crate: other tasks of this plan hold them.

## Steps

1. `translate`, `named`; `application.rs`'s sites.
2. `RestoreEdit`, `Answered`; `restore.rs`'s sites.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the counts; any site that stayed, and why.
